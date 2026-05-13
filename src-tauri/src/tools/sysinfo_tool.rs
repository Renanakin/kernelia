use super::ToolResult;
use serde_json::json;
use sysinfo::System;

/// Estructura de datos para el sistema (usada para el TelemetryPanel)
pub fn get_system_info_json() -> String {
    let mut sys = System::new_all();
    sys.refresh_all();

    let os_name = System::name().unwrap_or_else(|| "Unknown".into());
    let os_version = System::os_version().unwrap_or_else(|| "Unknown".into());
    let kernel = System::kernel_version().unwrap_or_else(|| "Unknown".into());
    let hostname = System::host_name().unwrap_or_else(|| "Unknown".into());

    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();

    let cpu_count = sys.cpus().len();
    let cpu_usage: f32 = if cpu_count > 0 {
        sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / cpu_count as f32
    } else {
        0.0
    };

    // Información de discos
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let mut disk_list = Vec::new();
    for disk in disks.list() {
        let total = disk.total_space();
        let available = disk.available_space();
        disk_list.push(json!({
            "mount_point": disk.mount_point().to_string_lossy(),
            "total_space": total,
            "available_space": available,
            "used_space": total - available,
            "file_system": disk.file_system().to_string_lossy()
        }));
    }

    let data = json!({
        "hostname": hostname,
        "os_name": os_name,
        "os_version": os_version,
        "kernel": kernel,
        "cpu_usage": cpu_usage,
        "cpu_count": cpu_count,
        "memory_total": total_memory,
        "memory_used": used_memory,
        "uptime": System::uptime(),
        "disks": disk_list
    });

    data.to_string()
}

/// Obtiene información detallada del sistema (formato texto para IA)
pub fn get_system_info() -> ToolResult {
    let mut sys = System::new_all();
    sys.refresh_all();

    let os_name = System::name().unwrap_or_else(|| "Unknown".into());
    let os_version = System::os_version().unwrap_or_else(|| "Unknown".into());
    let kernel = System::kernel_version().unwrap_or_else(|| "Unknown".into());
    let hostname = System::host_name().unwrap_or_else(|| "Unknown".into());

    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let total_swap = sys.total_swap();
    let used_swap = sys.used_swap();

    let cpu_count = sys.cpus().len();
    let cpu_name = sys
        .cpus()
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown".into());
    let cpu_usage: f32 = if cpu_count > 0 {
        sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / cpu_count as f32
    } else {
        0.0
    };

    // Información de discos
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let mut disk_info = Vec::new();
    for disk in disks.list() {
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total - available;
        let usage_pct = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        disk_info.push(format!(
            "  {} ({:?}): {:.1} GB / {:.1} GB ({:.0}% used)",
            disk.mount_point().display(),
            disk.file_system(),
            used as f64 / 1_073_741_824.0,
            total as f64 / 1_073_741_824.0,
            usage_pct
        ));
    }

    let info = format!(
        r#"System Information

Host: {}
OS: {} {}
Kernel: {}

CPU: {} ({} cores)
CPU Usage: {:.1}%

RAM: {:.1} GB / {:.1} GB ({:.0}% used)
Swap: {:.1} GB / {:.1} GB

Disks:
{}

Uptime: {} seconds"#,
        hostname,
        os_name,
        os_version,
        kernel,
        cpu_name,
        cpu_count,
        cpu_usage,
        used_memory as f64 / 1_073_741_824.0,
        total_memory as f64 / 1_073_741_824.0,
        if total_memory > 0 {
            (used_memory as f64 / total_memory as f64) * 100.0
        } else {
            0.0
        },
        used_swap as f64 / 1_073_741_824.0,
        total_swap as f64 / 1_073_741_824.0,
        disk_info.join("\n"),
        System::uptime()
    );

    ToolResult {
        tool_name: "get_system_info".to_string(),
        success: true,
        output: info,
        error: None,
    }
}

/// Resumen ejecutivo de unidades de almacenamiento para respuestas al usuario.
pub fn get_storage_summary() -> ToolResult {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let disk_count = disks.list().len();
    let mut total_bytes: u64 = 0;
    let mut used_bytes: u64 = 0;
    let mut lines = Vec::new();
    let mut warnings = Vec::new();

    for disk in disks.list() {
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total.saturating_sub(available);
        let usage_pct = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        let mount = disk.mount_point().display().to_string();
        total_bytes = total_bytes.saturating_add(total);
        used_bytes = used_bytes.saturating_add(used);

        if usage_pct >= 85.0 {
            warnings.push(format!(
                "{} esta al {:.0}% de uso; conviene liberar espacio o revisar respaldo.",
                mount, usage_pct
            ));
        }

        lines.push(format!(
            "- {} {}: {:.1} GB usados de {:.1} GB ({:.0}% usado, {:.1} GB libres)",
            mount,
            disk.file_system().to_string_lossy(),
            used as f64 / 1_073_741_824.0,
            total as f64 / 1_073_741_824.0,
            usage_pct,
            available as f64 / 1_073_741_824.0
        ));
    }

    let available_bytes = total_bytes.saturating_sub(used_bytes);
    let total_tb = total_bytes as f64 / 1_099_511_627_776.0;
    let used_tb = used_bytes as f64 / 1_099_511_627_776.0;
    let free_tb = available_bytes as f64 / 1_099_511_627_776.0;

    let mut output = format!(
        "El equipo tiene {} unidades de almacenamiento detectadas.\n\nCapacidad total: {:.2} TB\nUsado: {:.2} TB\nLibre aproximado: {:.2} TB\n\nDetalle:\n{}",
        disk_count,
        total_tb,
        used_tb,
        free_tb,
        lines.join("\n")
    );

    if !warnings.is_empty() {
        output.push_str("\n\nAlertas:\n");
        output.push_str(&warnings.join("\n"));
    }

    ToolResult {
        tool_name: "get_storage_summary".to_string(),
        success: true,
        output,
        error: None,
    }
}
