use crate::tools::{network_diagnostic, processes, sysinfo_tool, ToolResult};
use chrono::Local;
use std::fs;

/// Genera un reporte técnico consolidado para soporte
pub fn generate_support_report() -> ToolResult {
    let now = Local::now();
    let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let mut report = String::new();
    report.push_str(&format!("# REPORTE TÉCNICO KERNEL IA - {}\n\n", timestamp));

    // 1. Información del Sistema
    report.push_str("## 1. ESTADO DEL SISTEMA\n");
    let sys_info = sysinfo_tool::get_system_info();
    report.push_str(&sys_info.output);
    report.push_str("\n\n");

    // 2. Diagnóstico de Red
    report.push_str("## 2. DIAGNÓSTICO DE RED\n");
    let net_info = network_diagnostic::run_network_diagnostic();
    report.push_str(&net_info.output);
    report.push_str("\n\n");

    // 3. Procesos (Top 10 por Memoria)
    report.push_str("## 3. PROCESOS PESADOS (TOP 10)\n");
    let proc_info = processes::list_processes("memory", 10);
    report.push_str(&proc_info.output);
    report.push_str("\n\n");

    // 4. Conclusión automática (Mock)
    report.push_str("## 4. RESUMEN DE SALUD\n");
    report.push_str("Estado general: ÓPTIMO\n");
    report.push_str("Acciones recomendadas: Ninguna inmediata.\n");

    // Guardar reporte en archivo local
    let filename = format!("reporte_hackteck_{}.md", now.format("%Y%m%d_%H%M%S"));
    let save_path = format!("./{}", filename);

    match fs::write(&save_path, &report) {
        Ok(_) => ToolResult {
            tool_name: "generate_support_report".into(),
            success: true,
            output: format!("Reporte generado y guardado en: {}\n\n{}", filename, report),
            error: None,
        },
        Err(e) => ToolResult {
            tool_name: "generate_support_report".into(),
            success: false,
            output: report,
            error: Some(format!("Error al guardar el archivo: {}", e)),
        },
    }
}
