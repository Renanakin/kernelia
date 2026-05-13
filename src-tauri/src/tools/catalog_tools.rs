use crate::tools::{
    audit, cleanup, drivers, network_diagnostic, processes, rbac, sysinfo_tool, terminal,
    ToolDefinition, ToolResult,
};
use serde_json::json;
use std::fs;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn run_powershell(command: &str) -> Result<String, String> {
    let mut cmd = Command::new("powershell.exe");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", command]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let output = cmd
        .output()
        .map_err(|e| format!("Error ejecutando PowerShell: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn run_cmd(program: &str, args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new(program);
    cmd.args(args);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let output = cmd
        .output()
        .map_err(|e| format!("Error ejecutando {}: {}", program, e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn get_sys_value() -> serde_json::Value {
    serde_json::from_str::<serde_json::Value>(&sysinfo_tool::get_system_info_json())
        .unwrap_or_else(|_| json!({}))
}

fn require_str_arg<'a>(args: &'a serde_json::Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| format!("Parametro {} requerido", key))
}

fn folder_size_bytes(path: &Path) -> Result<u64, String> {
    let mut total = 0u64;
    let entries = fs::read_dir(path).map_err(|e| format!("Error leyendo directorio: {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Error leyendo entrada: {}", e))?;
        let p = entry.path();
        if p.is_dir() {
            total = total.saturating_add(folder_size_bytes(&p)?);
        } else {
            let meta = entry
                .metadata()
                .map_err(|e| format!("Error leyendo metadatos: {}", e))?;
            total = total.saturating_add(meta.len());
        }
    }
    Ok(total)
}

pub fn tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "get_os_info".into(),
            description: "Obtiene informacion del sistema operativo.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_hostname".into(),
            description: "Obtiene el hostname del equipo.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_current_user".into(),
            description: "Obtiene el usuario actual del sistema.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_uptime".into(),
            description: "Obtiene el tiempo de encendido del sistema.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_cpu_info".into(),
            description: "Obtiene informacion de CPU.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_memory_info".into(),
            description: "Obtiene informacion de memoria.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_disk_info".into(),
            description: "Obtiene informacion de discos.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_gpu_info".into(),
            description: "Obtiene informacion de GPU.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_battery_info".into(),
            description: "Obtiene informacion de bateria.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_local_ip".into(),
            description: "Obtiene la IP local del equipo.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_network_adapters".into(),
            description: "Lista adaptadores de red del equipo.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_environment_info".into(),
            description: "Obtiene informacion de entorno del sistema.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_cpu_usage".into(),
            description: "Obtiene uso actual de CPU.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_memory_usage".into(),
            description: "Obtiene uso actual de memoria.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_disk_usage".into(),
            description: "Obtiene uso actual de discos.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_network_usage".into(),
            description: "Obtiene resumen de uso de red.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_top_processes".into(),
            description: "Obtiene procesos con mayor consumo.".into(),
            parameters: json!({"type":"object","properties":{"sort_by":{"type":"string"},"limit":{"type":"integer"}},"required":[]}),
        },
        ToolDefinition {
            name: "get_running_services".into(),
            description: "Lista servicios en ejecucion.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_startup_programs".into(),
            description: "Lista programas de inicio.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_installed_programs".into(),
            description: "Lista programas instalados.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_windows_updates_status".into(),
            description: "Obtiene estado del servicio de Windows Update.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "ping_host".into(),
            description: "Ejecuta ping a un host.".into(),
            parameters: json!({"type":"object","properties":{"host":{"type":"string"},"count":{"type":"integer"}},"required":["host"]}),
        },
        ToolDefinition {
            name: "traceroute_host".into(),
            description: "Ejecuta traceroute a un host.".into(),
            parameters: json!({"type":"object","properties":{"host":{"type":"string"}},"required":["host"]}),
        },
        ToolDefinition {
            name: "dns_lookup".into(),
            description: "Realiza consulta DNS.".into(),
            parameters: json!({"type":"object","properties":{"host":{"type":"string"}},"required":["host"]}),
        },
        ToolDefinition {
            name: "test_tcp_port".into(),
            description: "Prueba conectividad a puerto TCP.".into(),
            parameters: json!({"type":"object","properties":{"host":{"type":"string"},"port":{"type":"integer"}},"required":["host","port"]}),
        },
        ToolDefinition {
            name: "get_wifi_info".into(),
            description: "Obtiene informacion de WiFi.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_default_gateway".into(),
            description: "Obtiene gateway por defecto.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_dns_servers".into(),
            description: "Obtiene servidores DNS configurados.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "flush_dns_cache".into(),
            description: "Limpia cache DNS del sistema.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "renew_ip_config".into(),
            description: "Renueva configuracion IP.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "release_ip_config".into(),
            description: "Libera configuracion IP.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "reset_network_stack".into(),
            description: "Resetea stack de red (winsock).".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_process_detail".into(),
            description: "Obtiene detalle de proceso por PID o nombre.".into(),
            parameters: json!({"type":"object","properties":{"pid":{"type":"integer"},"name":{"type":"string"}},"required":[]}),
        },
        ToolDefinition {
            name: "restart_process".into(),
            description: "Reinicia proceso por nombre usando ruta opcional.".into(),
            parameters: json!({"type":"object","properties":{"name":{"type":"string"},"path":{"type":"string"}},"required":["name"]}),
        },
        ToolDefinition {
            name: "find_high_cpu_processes".into(),
            description: "Busca procesos con alto consumo de CPU.".into(),
            parameters: json!({"type":"object","properties":{"limit":{"type":"integer"}},"required":[]}),
        },
        ToolDefinition {
            name: "find_high_memory_processes".into(),
            description: "Busca procesos con alto consumo de memoria.".into(),
            parameters: json!({"type":"object","properties":{"limit":{"type":"integer"}},"required":[]}),
        },
        ToolDefinition {
            name: "list_services".into(),
            description: "Lista servicios de Windows.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_service_status".into(),
            description: "Obtiene estado de un servicio.".into(),
            parameters: json!({"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}),
        },
        ToolDefinition {
            name: "start_service".into(),
            description: "Inicia un servicio de Windows.".into(),
            parameters: json!({"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}),
        },
        ToolDefinition {
            name: "stop_service".into(),
            description: "Detiene un servicio de Windows.".into(),
            parameters: json!({"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}),
        },
        ToolDefinition {
            name: "enable_service".into(),
            description: "Configura servicio en inicio automatico.".into(),
            parameters: json!({"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}),
        },
        ToolDefinition {
            name: "disable_service".into(),
            description: "Deshabilita servicio de Windows.".into(),
            parameters: json!({"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}),
        },
        ToolDefinition {
            name: "clean_temp_files".into(),
            description: "Limpia archivos temporales.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "empty_recycle_bin".into(),
            description: "Vacia papelera de reciclaje.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "run_disk_cleanup".into(),
            description: "Ejecuta liberador de espacio.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "clear_browser_cache".into(),
            description: "Limpia cache de navegadores comunes.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "clear_windows_update_cache".into(),
            description: "Limpia cache de Windows Update.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "check_disk_health".into(),
            description: "Revisa salud SMART de discos.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "scan_system_files".into(),
            description: "Escaneo SFC de integridad.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "repair_system_files".into(),
            description: "Reparacion SFC de integridad.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "run_dism_health_check".into(),
            description: "Chequeo DISM de imagen.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "run_dism_restore_health".into(),
            description: "Reparacion DISM de imagen.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_firewall_status".into(),
            description: "Estado de perfiles firewall.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "list_firewall_rules".into(),
            description: "Lista reglas de firewall.".into(),
            parameters: json!({"type":"object","properties":{"limit":{"type":"integer"}},"required":[]}),
        },
        ToolDefinition {
            name: "enable_firewall".into(),
            description: "Habilita firewall en todos los perfiles.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "disable_firewall".into(),
            description: "Deshabilita firewall en todos los perfiles.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_defender_status".into(),
            description: "Estado de Microsoft Defender.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "run_defender_quick_scan".into(),
            description: "Lanza escaneo rapido de Defender.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "run_defender_full_scan".into(),
            description: "Lanza escaneo completo de Defender.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_antivirus_status".into(),
            description: "Estado de antivirus registrados.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_security_center_status".into(),
            description: "Estado de Security Center.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "list_open_ports".into(),
            description: "Lista puertos abiertos/listening.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "list_listening_connections".into(),
            description: "Lista conexiones en escucha.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "list_active_connections".into(),
            description: "Lista conexiones activas.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "list_devices".into(),
            description: "Lista dispositivos PnP.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "list_problem_devices".into(),
            description: "Lista dispositivos con error.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_device_detail".into(),
            description: "Detalle de dispositivo por nombre.".into(),
            parameters: json!({"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}),
        },
        ToolDefinition {
            name: "get_driver_info".into(),
            description: "Informacion de driver por nombre.".into(),
            parameters: json!({"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}),
        },
        ToolDefinition {
            name: "update_driver".into(),
            description: "Abre panel para actualizar drivers opcionales.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "open_optional_driver_updates".into(),
            description: "Abre updates opcionales de drivers.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "rescan_devices".into(),
            description: "Re-escanea dispositivos plug and play.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_file_info".into(),
            description: "Obtiene metadatos de archivo/carpeta.".into(),
            parameters: json!({"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}),
        },
        ToolDefinition {
            name: "search_files".into(),
            description: "Busca archivos por patron.".into(),
            parameters: json!({"type":"object","properties":{"path":{"type":"string"},"pattern":{"type":"string"}},"required":["path","pattern"]}),
        },
        ToolDefinition {
            name: "create_folder".into(),
            description: "Crea carpeta.".into(),
            parameters: json!({"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}),
        },
        ToolDefinition {
            name: "delete_file".into(),
            description: "Elimina archivo o carpeta.".into(),
            parameters: json!({"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}),
        },
        ToolDefinition {
            name: "move_file".into(),
            description: "Mueve archivo o carpeta.".into(),
            parameters: json!({"type":"object","properties":{"source":{"type":"string"},"destination":{"type":"string"}},"required":["source","destination"]}),
        },
        ToolDefinition {
            name: "copy_file".into(),
            description: "Copia archivo o carpeta.".into(),
            parameters: json!({"type":"object","properties":{"source":{"type":"string"},"destination":{"type":"string"}},"required":["source","destination"]}),
        },
        ToolDefinition {
            name: "rename_file".into(),
            description: "Renombra archivo o carpeta.".into(),
            parameters: json!({"type":"object","properties":{"source":{"type":"string"},"destination":{"type":"string"}},"required":["source","destination"]}),
        },
        ToolDefinition {
            name: "calculate_folder_size".into(),
            description: "Calcula tamano de carpeta.".into(),
            parameters: json!({"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}),
        },
        ToolDefinition {
            name: "read_event_logs".into(),
            description: "Lee logs de eventos con filtro.".into(),
            parameters: json!({"type":"object","properties":{"log_name":{"type":"string"},"limit":{"type":"integer"}},"required":[]}),
        },
        ToolDefinition {
            name: "read_system_log".into(),
            description: "Lee log System.".into(),
            parameters: json!({"type":"object","properties":{"limit":{"type":"integer"}},"required":[]}),
        },
        ToolDefinition {
            name: "read_application_log".into(),
            description: "Lee log Application.".into(),
            parameters: json!({"type":"object","properties":{"limit":{"type":"integer"}},"required":[]}),
        },
        ToolDefinition {
            name: "read_security_log".into(),
            description: "Lee log Security.".into(),
            parameters: json!({"type":"object","properties":{"limit":{"type":"integer"}},"required":[]}),
        },
        ToolDefinition {
            name: "export_event_logs".into(),
            description: "Exporta eventos a archivo JSON.".into(),
            parameters: json!({"type":"object","properties":{"log_name":{"type":"string"},"path":{"type":"string"},"limit":{"type":"integer"}},"required":["log_name","path"]}),
        },
        ToolDefinition {
            name: "get_kernelia_audit_log".into(),
            description: "Devuelve auditoria interna KernelIA.".into(),
            parameters: json!({"type":"object","properties":{"limit":{"type":"integer"}},"required":[]}),
        },
        ToolDefinition {
            name: "search_kernelia_audit_log".into(),
            description: "Busca texto en auditoria interna.".into(),
            parameters: json!({"type":"object","properties":{"query":{"type":"string"},"limit":{"type":"integer"}},"required":["query"]}),
        },
        ToolDefinition {
            name: "get_power_plan".into(),
            description: "Obtiene plan de energia activo.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "set_power_plan".into(),
            description: "Establece plan de energia por GUID o alias.".into(),
            parameters: json!({"type":"object","properties":{"plan":{"type":"string"}},"required":["plan"]}),
        },
        ToolDefinition {
            name: "list_power_plans".into(),
            description: "Lista planes de energia.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_sleep_settings".into(),
            description: "Consulta timeout de suspension.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "set_sleep_settings".into(),
            description: "Define timeout AC/DC de suspension.".into(),
            parameters: json!({"type":"object","properties":{"ac_minutes":{"type":"integer"},"dc_minutes":{"type":"integer"}},"required":[]}),
        },
        ToolDefinition {
            name: "get_startup_impact".into(),
            description: "Lista impacto de apps de inicio.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "optimize_startup_apps".into(),
            description: "Deshabilita startup de alto impacto no-Microsoft.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "list_installed_apps".into(),
            description: "Lista aplicaciones instaladas.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "get_app_detail".into(),
            description: "Detalle de aplicacion instalada.".into(),
            parameters: json!({"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}),
        },
        ToolDefinition {
            name: "uninstall_app".into(),
            description: "Desinstala aplicacion por comando de uninstall.".into(),
            parameters: json!({"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}),
        },
        ToolDefinition {
            name: "check_app_updates".into(),
            description: "Busca actualizaciones via winget.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "list_windows_features".into(),
            description: "Lista features de Windows.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "enable_windows_feature".into(),
            description: "Habilita feature Windows.".into(),
            parameters: json!({"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}),
        },
        ToolDefinition {
            name: "disable_windows_feature".into(),
            description: "Deshabilita feature Windows.".into(),
            parameters: json!({"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}),
        },
        ToolDefinition {
            name: "run_shell_command".into(),
            description: "Ejecuta comando shell sensible.".into(),
            parameters: json!({"type":"object","properties":{"command":{"type":"string"}},"required":["command"]}),
        },
        ToolDefinition {
            name: "run_powershell_command".into(),
            description: "Ejecuta comando PowerShell sensible.".into(),
            parameters: json!({"type":"object","properties":{"command":{"type":"string"}},"required":["command"]}),
        },
        ToolDefinition {
            name: "edit_registry_key".into(),
            description: "Crea/edita clave valor de registro.".into(),
            parameters: json!({"type":"object","properties":{"path":{"type":"string"},"name":{"type":"string"},"value":{"type":"string"}},"required":["path","name","value"]}),
        },
        ToolDefinition {
            name: "delete_registry_key".into(),
            description: "Elimina valor de registro.".into(),
            parameters: json!({"type":"object","properties":{"path":{"type":"string"},"name":{"type":"string"}},"required":["path","name"]}),
        },
        ToolDefinition {
            name: "create_local_user".into(),
            description: "Crea usuario local.".into(),
            parameters: json!({"type":"object","properties":{"username":{"type":"string"},"password":{"type":"string"}},"required":["username","password"]}),
        },
        ToolDefinition {
            name: "delete_local_user".into(),
            description: "Elimina usuario local.".into(),
            parameters: json!({"type":"object","properties":{"username":{"type":"string"}},"required":["username"]}),
        },
        ToolDefinition {
            name: "reset_user_password".into(),
            description: "Resetea password de usuario local.".into(),
            parameters: json!({"type":"object","properties":{"username":{"type":"string"},"password":{"type":"string"}},"required":["username","password"]}),
        },
        ToolDefinition {
            name: "add_user_to_group".into(),
            description: "Agrega usuario a grupo local.".into(),
            parameters: json!({"type":"object","properties":{"username":{"type":"string"},"group":{"type":"string"}},"required":["username","group"]}),
        },
        ToolDefinition {
            name: "remove_user_from_group".into(),
            description: "Quita usuario de grupo local.".into(),
            parameters: json!({"type":"object","properties":{"username":{"type":"string"},"group":{"type":"string"}},"required":["username","group"]}),
        },
        ToolDefinition {
            name: "change_firewall_rule".into(),
            description: "Cambia estado de regla firewall.".into(),
            parameters: json!({"type":"object","properties":{"name":{"type":"string"},"enabled":{"type":"boolean"}},"required":["name","enabled"]}),
        },
        ToolDefinition {
            name: "change_network_adapter_config".into(),
            description: "Cambia DNS de adaptador de red.".into(),
            parameters: json!({"type":"object","properties":{"adapter":{"type":"string"},"dns1":{"type":"string"},"dns2":{"type":"string"}},"required":["adapter","dns1"]}),
        },
        ToolDefinition {
            name: "reboot_system".into(),
            description: "Reinicia sistema operativo.".into(),
            parameters: json!({"type":"object","properties":{"delay_seconds":{"type":"integer"}},"required":[]}),
        },
        ToolDefinition {
            name: "shutdown_system".into(),
            description: "Apaga sistema operativo.".into(),
            parameters: json!({"type":"object","properties":{"delay_seconds":{"type":"integer"}},"required":[]}),
        },
        ToolDefinition {
            name: "run_elevated_command".into(),
            description: "Ejecuta comando elevado (MegaBoss).".into(),
            parameters: json!({"type":"object","properties":{"command":{"type":"string"}},"required":["command"]}),
        },
        ToolDefinition {
            name: "force_kill_process".into(),
            description: "Fuerza terminacion de proceso.".into(),
            parameters: json!({"type":"object","properties":{"pid":{"type":"integer"},"name":{"type":"string"}},"required":[]}),
        },
        ToolDefinition {
            name: "force_delete_file".into(),
            description: "Fuerza borrado de archivo/carpeta.".into(),
            parameters: json!({"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}),
        },
        ToolDefinition {
            name: "modify_system_registry".into(),
            description: "Modifica registro en rama de sistema.".into(),
            parameters: json!({"type":"object","properties":{"path":{"type":"string"},"name":{"type":"string"},"value":{"type":"string"}},"required":["path","name","value"]}),
        },
        ToolDefinition {
            name: "disable_security_component".into(),
            description: "Deshabilita componente de seguridad.".into(),
            parameters: json!({"type":"object","properties":{"component":{"type":"string"}},"required":["component"]}),
        },
        ToolDefinition {
            name: "enable_security_component".into(),
            description: "Habilita componente de seguridad.".into(),
            parameters: json!({"type":"object","properties":{"component":{"type":"string"}},"required":["component"]}),
        },
        ToolDefinition {
            name: "reset_windows_network_stack".into(),
            description: "Resetea pila de red completa.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "repair_windows_image".into(),
            description: "Repara imagen de Windows con DISM.".into(),
            parameters: json!({"type":"object","properties":{},"required":[]}),
        },
        ToolDefinition {
            name: "execute_admin_script".into(),
            description: "Ejecuta script administrativo desde ruta.".into(),
            parameters: json!({"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}),
        },
    ]
}

pub fn is_catalog_tool(name: &str) -> bool {
    tool_definitions().iter().any(|d| d.name == name)
}

pub async fn execute_catalog_tool(name: &str, args: &serde_json::Value) -> ToolResult {
    let sys = get_sys_value();

    match name {
        "get_os_info" => ToolResult { tool_name: name.into(), success: true, output: json!({"os_name": sys["os_name"], "os_version": sys["os_version"], "kernel_version": sys["kernel_version"]}).to_string(), error: None },
        "get_hostname" => ToolResult { tool_name: name.into(), success: true, output: json!({"hostname": sys["hostname"]}).to_string(), error: None },
        "get_current_user" => ToolResult { tool_name: name.into(), success: true, output: json!({"user": sys["current_user"]}).to_string(), error: None },
        "get_uptime" => ToolResult { tool_name: name.into(), success: true, output: json!({"uptime_seconds": sys["uptime_seconds"]}).to_string(), error: None },
        "get_cpu_info" => ToolResult { tool_name: name.into(), success: true, output: json!({"cpu_cores": sys["cpu_cores"], "cpu_usage": sys["cpu_usage"]}).to_string(), error: None },
        "get_memory_info" => ToolResult { tool_name: name.into(), success: true, output: json!({"memory_total": sys["memory_total"], "memory_used": sys["memory_used"]}).to_string(), error: None },
        "get_disk_info" => ToolResult { tool_name: name.into(), success: true, output: json!({"disks": sys["disks"]}).to_string(), error: None },
        "get_gpu_info" => ToolResult { tool_name: name.into(), success: true, output: json!({"gpu": sys["gpu"]}).to_string(), error: None },
        "get_battery_info" => ToolResult { tool_name: name.into(), success: true, output: json!({"battery": sys["battery"]}).to_string(), error: None },
        "get_local_ip" => {
            let result = run_powershell("(Get-NetIPAddress -AddressFamily IPv4 | Where-Object { $_.IPAddress -notlike '169.254*' -and $_.IPAddress -ne '127.0.0.1' } | Select-Object -First 1 -ExpandProperty IPAddress)");
            match result {
                Ok(ip) => ToolResult { tool_name: name.into(), success: true, output: json!({"local_ip": ip}).to_string(), error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "get_network_adapters" => {
            let result = run_powershell("Get-NetAdapter | Select-Object Name, Status, LinkSpeed | ConvertTo-Json -Depth 3");
            match result {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "get_environment_info" => {
            let result = run_powershell("Get-ChildItem Env: | Sort-Object Name | Select-Object Name, Value | ConvertTo-Json -Depth 3");
            match result {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "get_cpu_usage" => ToolResult { tool_name: name.into(), success: true, output: json!({"cpu_usage": sys["cpu_usage"]}).to_string(), error: None },
        "get_memory_usage" => ToolResult { tool_name: name.into(), success: true, output: json!({"memory_total": sys["memory_total"], "memory_used": sys["memory_used"]}).to_string(), error: None },
        "get_disk_usage" => ToolResult { tool_name: name.into(), success: true, output: json!({"disks": sys["disks"]}).to_string(), error: None },
        "get_network_usage" => {
            let diag = network_diagnostic::run_network_diagnostic_json();
            ToolResult { tool_name: name.into(), success: true, output: diag, error: None }
        }
        "get_top_processes" => {
            let sort_by = args.get("sort_by").and_then(|v| v.as_str()).unwrap_or("cpu");
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
            processes::list_processes(sort_by, limit)
        }
        "get_running_services" => {
            let result = run_powershell("Get-Service | Where-Object Status -eq 'Running' | Select-Object Name, Status, DisplayName | ConvertTo-Json -Depth 3");
            match result {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "get_startup_programs" => {
            let result = run_powershell("Get-CimInstance Win32_StartupCommand | Select-Object Name, Command, Location | ConvertTo-Json -Depth 3");
            match result {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "get_installed_programs" => {
            let result = run_powershell("Get-ItemProperty HKLM:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\* , HKLM:\\Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\* | Select-Object DisplayName, DisplayVersion, Publisher | Where-Object { $_.DisplayName } | ConvertTo-Json -Depth 3");
            match result {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "get_windows_updates_status" => {
            let result = run_powershell("Get-Service wuauserv | Select-Object Name, Status, StartType | ConvertTo-Json -Depth 3");
            match result {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "ping_host" => {
            let host = args.get("host").and_then(|v| v.as_str()).unwrap_or("");
            let count = args.get("count").and_then(|v| v.as_u64()).unwrap_or(4).to_string();
            if host.is_empty() {
                return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some("Parametro host requerido".to_string()) };
            }
            match run_cmd("ping", &["-n", &count, host]) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "traceroute_host" => {
            let host = args.get("host").and_then(|v| v.as_str()).unwrap_or("");
            if host.is_empty() {
                return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some("Parametro host requerido".to_string()) };
            }
            match run_cmd("tracert", &[host]) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "dns_lookup" => {
            let host = args.get("host").and_then(|v| v.as_str()).unwrap_or("");
            if host.is_empty() {
                return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some("Parametro host requerido".to_string()) };
            }
            match run_cmd("nslookup", &[host]) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "test_tcp_port" => {
            let host = args.get("host").and_then(|v| v.as_str()).unwrap_or("");
            let port = args.get("port").and_then(|v| v.as_u64()).unwrap_or(0);
            if host.is_empty() || port == 0 {
                return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some("Parametros host y port requeridos".to_string()) };
            }
            let ps = format!("Test-NetConnection -ComputerName '{}' -Port {} | Select-Object ComputerName, RemotePort, TcpTestSucceeded | ConvertTo-Json -Depth 3", host, port);
            match run_powershell(&ps) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "get_wifi_info" => match run_cmd("netsh", &["wlan", "show", "interfaces"]) {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "get_default_gateway" => {
            let result = run_powershell("Get-NetRoute -DestinationPrefix '0.0.0.0/0' | Select-Object -First 1 -ExpandProperty NextHop");
            match result {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: json!({"default_gateway": out}).to_string(), error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "get_dns_servers" => {
            let result = run_powershell("Get-DnsClientServerAddress -AddressFamily IPv4 | Select-Object InterfaceAlias, ServerAddresses | ConvertTo-Json -Depth 4");
            match result {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "flush_dns_cache" => match run_cmd("ipconfig", &["/flushdns"]) {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "renew_ip_config" => match run_cmd("ipconfig", &["/renew"]) {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "release_ip_config" => match run_cmd("ipconfig", &["/release"]) {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "reset_network_stack" => match run_cmd("netsh", &["winsock", "reset"]) {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "get_process_detail" => {
            let pid = args.get("pid").and_then(|v| v.as_u64()).map(|v| v as u32);
            let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let result = processes::list_processes("cpu", 500);
            if !result.success {
                return result;
            }
            let parsed = serde_json::from_str::<serde_json::Value>(&result.output).unwrap_or_else(|_| json!([]));
            let found = parsed
                .as_array()
                .and_then(|rows| {
                    rows.iter().find(|row| {
                        let pid_match = pid.map(|p| row["pid"].as_u64() == Some(p as u64)).unwrap_or(false);
                        let name_match = !name.is_empty()
                            && row["name"]
                                .as_str()
                                .map(|n| n.eq_ignore_ascii_case(name))
                                .unwrap_or(false);
                        pid_match || name_match
                    })
                })
                .cloned();

            match found {
                Some(row) => ToolResult { tool_name: name.into(), success: true, output: row.to_string(), error: None },
                None => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some("Proceso no encontrado".to_string()) },
            }
        }
        "restart_process" => {
            let proc_name = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
            if proc_name.is_empty() {
                return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some("Parametro name requerido".to_string()) };
            }
            let _ = processes::kill_process(None, Some(proc_name));
            if !path.is_empty() {
                let ps = format!("Start-Process -FilePath '{}'", path);
                match run_powershell(&ps) {
                    Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                    Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
                }
            } else {
                ToolResult {
                    tool_name: name.into(),
                    success: true,
                    output: "Proceso terminado. Para reinicio automatico, provee parametro path.".to_string(),
                    error: None,
                }
            }
        }
        "find_high_cpu_processes" => {
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
            processes::list_processes("cpu", limit)
        }
        "find_high_memory_processes" => {
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
            processes::list_processes("memory", limit)
        }
        "list_services" => {
            let result = run_powershell("Get-Service | Select-Object Name, Status, StartType, DisplayName | ConvertTo-Json -Depth 3");
            match result {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "get_service_status" => {
            let service = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if service.is_empty() {
                return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some("Parametro name requerido".to_string()) };
            }
            let ps = format!("Get-Service -Name '{}' | Select-Object Name, Status, StartType | ConvertTo-Json -Depth 3", service);
            match run_powershell(&ps) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "start_service" => {
            let service = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if service.is_empty() {
                return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some("Parametro name requerido".to_string()) };
            }
            let ps = format!("Start-Service -Name '{}' ; Get-Service -Name '{}' | Select-Object Name, Status | ConvertTo-Json -Depth 3", service, service);
            match run_powershell(&ps) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "stop_service" => {
            let service = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if service.is_empty() {
                return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some("Parametro name requerido".to_string()) };
            }
            let ps = format!("Stop-Service -Name '{}' -Force ; Get-Service -Name '{}' | Select-Object Name, Status | ConvertTo-Json -Depth 3", service, service);
            match run_powershell(&ps) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "enable_service" => {
            let service = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if service.is_empty() {
                return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some("Parametro name requerido".to_string()) };
            }
            let ps = format!("Set-Service -Name '{}' -StartupType Automatic ; Get-Service -Name '{}' | Select-Object Name, StartType | ConvertTo-Json -Depth 3", service, service);
            match run_powershell(&ps) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "disable_service" => {
            let service = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if service.is_empty() {
                return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some("Parametro name requerido".to_string()) };
            }
            let ps = format!("Set-Service -Name '{}' -StartupType Disabled ; Get-Service -Name '{}' | Select-Object Name, StartType | ConvertTo-Json -Depth 3", service, service);
            match run_powershell(&ps) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "clean_temp_files" => cleanup::run_cleanup(Some(vec!["temp".to_string()])).await,
        "empty_recycle_bin" => cleanup::run_cleanup(Some(vec!["recycle".to_string()])).await,
        "run_disk_cleanup" => match run_cmd("cleanmgr", &["/VERYLOWDISK"]) {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "clear_browser_cache" => {
            let script = r#"
$paths = @(
  "$env:LOCALAPPDATA\\Google\\Chrome\\User Data\\Default\\Cache",
  "$env:LOCALAPPDATA\\Microsoft\\Edge\\User Data\\Default\\Cache",
  "$env:LOCALAPPDATA\\Mozilla\\Firefox\\Profiles"
)
$deleted = @()
foreach ($p in $paths) { if (Test-Path $p) { Remove-Item -Path $p -Recurse -Force -ErrorAction SilentlyContinue; $deleted += $p } }
$deleted | ConvertTo-Json -Compress
"#;
            match run_powershell(script) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "clear_windows_update_cache" => {
            let script = "Stop-Service wuauserv -ErrorAction SilentlyContinue; Remove-Item -Path 'C:\\Windows\\SoftwareDistribution\\Download\\*' -Recurse -Force -ErrorAction SilentlyContinue; Start-Service wuauserv -ErrorAction SilentlyContinue; 'OK'";
            match run_powershell(script) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "check_disk_health" => match run_powershell("Get-PhysicalDisk | Select-Object FriendlyName,HealthStatus,OperationalStatus,Size | ConvertTo-Json -Depth 3") {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "scan_system_files" => match run_cmd("sfc", &["/verifyonly"]) {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "repair_system_files" => match run_cmd("sfc", &["/scannow"]) {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "run_dism_health_check" => match run_cmd("DISM", &["/Online", "/Cleanup-Image", "/CheckHealth"]) {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "run_dism_restore_health" => match run_cmd("DISM", &["/Online", "/Cleanup-Image", "/RestoreHealth"]) {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "get_firewall_status" => match run_powershell("Get-NetFirewallProfile | Select-Object Name, Enabled, DefaultInboundAction, DefaultOutboundAction | ConvertTo-Json -Depth 3") {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "list_firewall_rules" => {
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(200);
            let ps = format!("Get-NetFirewallRule | Select-Object -First {} DisplayName,Enabled,Direction,Action,Profile | ConvertTo-Json -Depth 3", limit);
            match run_powershell(&ps) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "enable_firewall" => match run_powershell("Set-NetFirewallProfile -Profile Domain,Public,Private -Enabled True; 'OK'") {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "disable_firewall" => match run_powershell("Set-NetFirewallProfile -Profile Domain,Public,Private -Enabled False; 'OK'") {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "get_defender_status" => match run_powershell("Get-MpComputerStatus | Select-Object AMServiceEnabled,AntispywareEnabled,AntivirusEnabled,RealTimeProtectionEnabled,QuickScanStartTime,FullScanStartTime | ConvertTo-Json -Depth 3") {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "run_defender_quick_scan" => match run_powershell("Start-MpScan -ScanType QuickScan; 'QuickScan launched'") {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "run_defender_full_scan" => match run_powershell("Start-MpScan -ScanType FullScan; 'FullScan launched'") {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "get_antivirus_status" => match run_powershell("Get-CimInstance -Namespace root/SecurityCenter2 -ClassName AntivirusProduct | Select-Object displayName,pathToSignedProductExe,productState | ConvertTo-Json -Depth 3") {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "get_security_center_status" => match run_powershell("Get-Service SecurityHealthService,wscsvc | Select-Object Name,Status,StartType | ConvertTo-Json -Depth 3") {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "list_open_ports" | "list_listening_connections" => match run_cmd("netstat", &["-ano"]) {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "list_active_connections" => match run_cmd("netstat", &["-anob"]) {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "list_devices" => match run_powershell("Get-PnpDevice | Select-Object Status,Class,FriendlyName,InstanceId | ConvertTo-Json -Depth 3") {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "list_problem_devices" => drivers::list_driver_issues(),
        "get_device_detail" => match require_str_arg(args, "name") {
            Ok(device_name) => {
                let ps = format!("Get-PnpDevice | Where-Object {{$_.FriendlyName -like '*{}*'}} | Select-Object Status,Class,FriendlyName,InstanceId | ConvertTo-Json -Depth 3", device_name.replace('\'', "''"));
                match run_powershell(&ps) {
                    Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                    Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
                }
            }
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "get_driver_info" => match require_str_arg(args, "name") {
            Ok(device_name) => {
                let ps = format!("Get-CimInstance Win32_PnPSignedDriver | Where-Object {{$_.DeviceName -like '*{}*'}} | Select-Object DeviceName,DriverVersion,DriverDate,Manufacturer,InfName | ConvertTo-Json -Depth 3", device_name.replace('\'', "''"));
                match run_powershell(&ps) {
                    Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                    Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
                }
            }
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "update_driver" | "open_optional_driver_updates" => drivers::search_missing_driver(),
        "rescan_devices" => match run_cmd("pnputil", &["/scan-devices"]) {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "get_file_info" => match require_str_arg(args, "path") {
            Ok(path) => {
                let p = Path::new(path);
                match fs::metadata(p) {
                    Ok(meta) => ToolResult {
                        tool_name: name.into(),
                        success: true,
                        output: json!({
                            "path": p.display().to_string(),
                            "is_dir": meta.is_dir(),
                            "is_file": meta.is_file(),
                            "size": meta.len(),
                            "readonly": meta.permissions().readonly(),
                        }).to_string(),
                        error: None,
                    },
                    Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(format!("Error leyendo metadatos: {}", e)) },
                }
            }
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "search_files" => {
            let base = match require_str_arg(args, "path") {
                Ok(v) => v,
                Err(e) => return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            };
            let pattern = match require_str_arg(args, "pattern") {
                Ok(v) => v.to_lowercase(),
                Err(e) => return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            };
            let mut matches = Vec::new();
            let script = format!("Get-ChildItem -Path '{}' -Recurse -ErrorAction SilentlyContinue | Where-Object {{$_.Name -like '*{}*'}} | Select-Object -First 200 FullName | ConvertTo-Json -Depth 3", base.replace('\'', "''"), pattern.replace('\'', "''"));
            match run_powershell(&script) {
                Ok(out) => {
                    matches.push(out);
                    ToolResult { tool_name: name.into(), success: true, output: matches.join("\n"), error: None }
                }
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "create_folder" => match require_str_arg(args, "path") {
            Ok(path) => match fs::create_dir_all(path) {
                Ok(_) => ToolResult { tool_name: name.into(), success: true, output: format!("Carpeta creada: {}", path), error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e.to_string()) },
            },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "delete_file" | "force_delete_file" => match require_str_arg(args, "path") {
            Ok(path) => {
                let p = Path::new(path);
                let res = if p.is_dir() { fs::remove_dir_all(p) } else { fs::remove_file(p) };
                match res {
                    Ok(_) => ToolResult { tool_name: name.into(), success: true, output: format!("Eliminado: {}", path), error: None },
                    Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e.to_string()) },
                }
            }
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "move_file" | "rename_file" => {
            let source = match require_str_arg(args, "source") {
                Ok(v) => v,
                Err(e) => return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            };
            let destination = match require_str_arg(args, "destination") {
                Ok(v) => v,
                Err(e) => return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            };
            match fs::rename(source, destination) {
                Ok(_) => ToolResult { tool_name: name.into(), success: true, output: format!("Movido: {} -> {}", source, destination), error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e.to_string()) },
            }
        }
        "copy_file" => {
            let source = match require_str_arg(args, "source") {
                Ok(v) => v,
                Err(e) => return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            };
            let destination = match require_str_arg(args, "destination") {
                Ok(v) => v,
                Err(e) => return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            };
            let src = Path::new(source);
            let res = if src.is_file() {
                fs::copy(source, destination).map(|_| ())
            } else {
                Err(std::io::Error::other("Solo soporta copia de archivo"))
            };
            match res {
                Ok(_) => ToolResult { tool_name: name.into(), success: true, output: format!("Copiado: {} -> {}", source, destination), error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e.to_string()) },
            }
        }
        "calculate_folder_size" => match require_str_arg(args, "path") {
            Ok(path) => {
                let p = Path::new(path);
                match folder_size_bytes(p) {
                    Ok(size) => ToolResult { tool_name: name.into(), success: true, output: json!({"path": path, "size_bytes": size}).to_string(), error: None },
                    Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
                }
            }
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "read_event_logs" => {
            let log_name = args.get("log_name").and_then(|v| v.as_str()).unwrap_or("System");
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(100);
            let ps = format!("Get-WinEvent -LogName '{}' -MaxEvents {} | Select-Object TimeCreated,Id,LevelDisplayName,ProviderName,Message | ConvertTo-Json -Depth 4", log_name.replace('\'', "''"), limit);
            match run_powershell(&ps) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "read_system_log" => {
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(100);
            let ps = format!("Get-WinEvent -LogName 'System' -MaxEvents {} | Select-Object TimeCreated,Id,LevelDisplayName,ProviderName,Message | ConvertTo-Json -Depth 4", limit);
            match run_powershell(&ps) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "read_application_log" => {
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(100);
            let ps = format!("Get-WinEvent -LogName 'Application' -MaxEvents {} | Select-Object TimeCreated,Id,LevelDisplayName,ProviderName,Message | ConvertTo-Json -Depth 4", limit);
            match run_powershell(&ps) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "read_security_log" => {
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(100);
            let ps = format!("Get-WinEvent -LogName 'Security' -MaxEvents {} | Select-Object TimeCreated,Id,LevelDisplayName,ProviderName,Message | ConvertTo-Json -Depth 4", limit);
            match run_powershell(&ps) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "export_event_logs" => {
            let log_name = match require_str_arg(args, "log_name") {
                Ok(v) => v,
                Err(e) => return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            };
            let path = match require_str_arg(args, "path") {
                Ok(v) => v,
                Err(e) => return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            };
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(200);
            let ps = format!("Get-WinEvent -LogName '{}' -MaxEvents {} | Select-Object TimeCreated,Id,LevelDisplayName,ProviderName,Message | ConvertTo-Json -Depth 4 | Out-File -FilePath '{}' -Encoding utf8; 'OK'", log_name.replace('\'', "''"), limit, path.replace('\'', "''"));
            match run_powershell(&ps) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "get_kernelia_audit_log" => {
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
            let entries = audit::read_audit_logs(limit);
            ToolResult { tool_name: name.into(), success: true, output: serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string()), error: None }
        }
        "search_kernelia_audit_log" => {
            let q = match require_str_arg(args, "query") {
                Ok(v) => v.to_lowercase(),
                Err(e) => return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            };
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(200) as usize;
            let filtered: Vec<_> = audit::read_audit_logs(limit)
                .into_iter()
                .filter(|e| {
                    e.tool.to_lowercase().contains(&q)
                        || e.action.to_lowercase().contains(&q)
                        || e.error.as_ref().map(|x| x.to_lowercase().contains(&q)).unwrap_or(false)
                })
                .collect();
            ToolResult { tool_name: name.into(), success: true, output: serde_json::to_string(&filtered).unwrap_or_else(|_| "[]".to_string()), error: None }
        }
        "get_power_plan" | "list_power_plans" => match run_cmd("powercfg", &["/LIST"]) {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "set_power_plan" => match require_str_arg(args, "plan") {
            Ok(plan) => {
                let value = match plan.to_lowercase().as_str() {
                    "balanced" => "381b4222-f694-41f0-9685-ff5bb260df2e",
                    "high_performance" | "alto_rendimiento" => "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c",
                    "power_saver" | "ahorro" => "a1841308-3541-4fab-bc81-f71556f20b4a",
                    _ => plan,
                };
                match run_cmd("powercfg", &["/SETACTIVE", value]) {
                    Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                    Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
                }
            }
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "get_sleep_settings" => match run_cmd("powercfg", &["/Q"]) {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "set_sleep_settings" => {
            let ac = args.get("ac_minutes").and_then(|v| v.as_u64()).unwrap_or(30).to_string();
            let dc = args.get("dc_minutes").and_then(|v| v.as_u64()).unwrap_or(15).to_string();
            let first = run_cmd("powercfg", &["/change", "standby-timeout-ac", &ac]);
            let second = run_cmd("powercfg", &["/change", "standby-timeout-dc", &dc]);
            match (first, second) {
                (Ok(a), Ok(b)) => ToolResult { tool_name: name.into(), success: true, output: format!("{}\n{}", a, b), error: None },
                (Err(e), _) | (_, Err(e)) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "get_startup_impact" => match run_powershell("Get-CimInstance Win32_StartupCommand | Select-Object Name,Command,Location,User | ConvertTo-Json -Depth 3") {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "optimize_startup_apps" => match run_powershell("Get-CimInstance Win32_StartupCommand | Where-Object { $_.Command -notmatch 'Microsoft' } | Select-Object Name,Command,Location | ConvertTo-Json -Depth 3") {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: format!("Sugerencias de optimizacion (revision manual): {}", out), error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "list_installed_apps" => {
            let result = run_powershell("Get-ItemProperty HKLM:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\* , HKLM:\\Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\* | Select-Object DisplayName, DisplayVersion, Publisher | Where-Object { $_.DisplayName } | ConvertTo-Json -Depth 3");
            match result {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "get_app_detail" => match require_str_arg(args, "name") {
            Ok(app_name) => {
                let ps = format!("Get-ItemProperty HKLM:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\* , HKLM:\\Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\* | Where-Object {{$_.DisplayName -like '*{}*'}} | Select-Object DisplayName,DisplayVersion,Publisher,InstallDate,UninstallString | ConvertTo-Json -Depth 3", app_name.replace('\'', "''"));
                match run_powershell(&ps) {
                    Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                    Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
                }
            }
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "uninstall_app" => match require_str_arg(args, "name") {
            Ok(app_name) => {
                let ps = format!("$app = Get-ItemProperty HKLM:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\* , HKLM:\\Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\* | Where-Object {{$_.DisplayName -like '*{}*'}} | Select-Object -First 1; if($app -and $app.UninstallString) {{ Start-Process -FilePath 'cmd.exe' -ArgumentList '/c', $app.UninstallString -Wait; 'UNINSTALL_STARTED' }} else {{ 'APP_OR_UNINSTALL_NOT_FOUND' }}", app_name.replace('\'', "''"));
                match run_powershell(&ps) {
                    Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                    Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
                }
            }
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "check_app_updates" => match run_cmd("winget", &["upgrade"]) {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "list_windows_features" => match run_cmd("DISM", &["/Online", "/Get-Features", "/Format:Table"]) {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "enable_windows_feature" => match require_str_arg(args, "name") {
            Ok(feature) => match run_cmd("DISM", &["/Online", "/Enable-Feature", &format!("/FeatureName:{}", feature), "/All"]) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "disable_windows_feature" => match require_str_arg(args, "name") {
            Ok(feature) => match run_cmd("DISM", &["/Online", "/Disable-Feature", &format!("/FeatureName:{}", feature)]) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "run_shell_command" => match require_str_arg(args, "command") {
            Ok(cmd) => terminal::run_command(cmd, None).await,
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "run_powershell_command" | "run_elevated_command" => match require_str_arg(args, "command") {
            Ok(cmd) => match run_powershell(cmd) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "edit_registry_key" | "modify_system_registry" => {
            let path = match require_str_arg(args, "path") { Ok(v) => v, Err(e) => return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) } };
            let reg_name = match require_str_arg(args, "name") { Ok(v) => v, Err(e) => return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) } };
            let value = match require_str_arg(args, "value") { Ok(v) => v, Err(e) => return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) } };
            let ps = format!("New-Item -Path '{}' -Force | Out-Null; New-ItemProperty -Path '{}' -Name '{}' -Value '{}' -PropertyType String -Force | Out-Null; 'OK'", path.replace('\'', "''"), path.replace('\'', "''"), reg_name.replace('\'', "''"), value.replace('\'', "''"));
            match run_powershell(&ps) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "delete_registry_key" => {
            let path = match require_str_arg(args, "path") { Ok(v) => v, Err(e) => return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) } };
            let reg_name = match require_str_arg(args, "name") { Ok(v) => v, Err(e) => return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) } };
            let ps = format!("Remove-ItemProperty -Path '{}' -Name '{}' -ErrorAction Stop; 'OK'", path.replace('\'', "''"), reg_name.replace('\'', "''"));
            match run_powershell(&ps) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "create_local_user" => {
            let username = match require_str_arg(args, "username") { Ok(v) => v, Err(e) => return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) } };
            let password = match require_str_arg(args, "password") { Ok(v) => v, Err(e) => return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) } };
            let ps = format!("$p=ConvertTo-SecureString '{}' -AsPlainText -Force; New-LocalUser -Name '{}' -Password $p -ErrorAction Stop; 'OK'", password.replace('\'', "''"), username.replace('\'', "''"));
            match run_powershell(&ps) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "delete_local_user" => {
            let username = match require_str_arg(args, "username") { Ok(v) => v, Err(e) => return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) } };
            let ps = format!("Remove-LocalUser -Name '{}' -ErrorAction Stop; 'OK'", username.replace('\'', "''"));
            match run_powershell(&ps) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "reset_user_password" => {
            let username = match require_str_arg(args, "username") { Ok(v) => v, Err(e) => return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) } };
            let password = match require_str_arg(args, "password") { Ok(v) => v, Err(e) => return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) } };
            let ps = format!("$p=ConvertTo-SecureString '{}' -AsPlainText -Force; Set-LocalUser -Name '{}' -Password $p -ErrorAction Stop; 'OK'", password.replace('\'', "''"), username.replace('\'', "''"));
            match run_powershell(&ps) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "add_user_to_group" | "remove_user_from_group" => {
            let username = match require_str_arg(args, "username") { Ok(v) => v, Err(e) => return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) } };
            let group = match require_str_arg(args, "group") { Ok(v) => v, Err(e) => return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) } };
            let op = if name == "add_user_to_group" { "Add-LocalGroupMember" } else { "Remove-LocalGroupMember" };
            let ps = format!("{} -Group '{}' -Member '{}' -ErrorAction Stop; 'OK'", op, group.replace('\'', "''"), username.replace('\'', "''"));
            match run_powershell(&ps) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "change_firewall_rule" => {
            let rule_name = match require_str_arg(args, "name") { Ok(v) => v, Err(e) => return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) } };
            let enabled = args.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
            let ps = format!("Set-NetFirewallRule -DisplayName '{}' -Enabled {}; 'OK'", rule_name.replace('\'', "''"), if enabled { "True" } else { "False" });
            match run_powershell(&ps) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "change_network_adapter_config" => {
            let adapter = match require_str_arg(args, "adapter") { Ok(v) => v, Err(e) => return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) } };
            let dns1 = match require_str_arg(args, "dns1") { Ok(v) => v, Err(e) => return ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) } };
            let dns2 = args.get("dns2").and_then(|v| v.as_str()).unwrap_or("");
            let ps = if dns2.is_empty() {
                format!("Set-DnsClientServerAddress -InterfaceAlias '{}' -ServerAddresses @('{}'); 'OK'", adapter.replace('\'', "''"), dns1.replace('\'', "''"))
            } else {
                format!("Set-DnsClientServerAddress -InterfaceAlias '{}' -ServerAddresses @('{}','{}'); 'OK'", adapter.replace('\'', "''"), dns1.replace('\'', "''"), dns2.replace('\'', "''"))
            };
            match run_powershell(&ps) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "reboot_system" => {
            let delay = args.get("delay_seconds").and_then(|v| v.as_u64()).unwrap_or(0).to_string();
            match run_cmd("shutdown", &["/r", "/t", &delay]) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "shutdown_system" => {
            let delay = args.get("delay_seconds").and_then(|v| v.as_u64()).unwrap_or(0).to_string();
            match run_cmd("shutdown", &["/s", "/t", &delay]) {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            }
        }
        "force_kill_process" => {
            let pid = args.get("pid").and_then(|v| v.as_u64()).map(|p| p as u32);
            let pname = args.get("name").and_then(|v| v.as_str());
            processes::kill_process(pid, pname)
        }
        "disable_security_component" => match require_str_arg(args, "component") {
            Ok(c) if c.eq_ignore_ascii_case("firewall") => match run_powershell("Set-NetFirewallProfile -Profile Domain,Public,Private -Enabled False; 'OK'") {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            },
            Ok(c) if c.eq_ignore_ascii_case("defender_realtime") => match run_powershell("Set-MpPreference -DisableRealtimeMonitoring $true; 'OK'") {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            },
            Ok(c) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(format!("Componente no soportado: {}", c)) },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "enable_security_component" => match require_str_arg(args, "component") {
            Ok(c) if c.eq_ignore_ascii_case("firewall") => match run_powershell("Set-NetFirewallProfile -Profile Domain,Public,Private -Enabled True; 'OK'") {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            },
            Ok(c) if c.eq_ignore_ascii_case("defender_realtime") => match run_powershell("Set-MpPreference -DisableRealtimeMonitoring $false; 'OK'") {
                Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
            },
            Ok(c) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(format!("Componente no soportado: {}", c)) },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "reset_windows_network_stack" => match run_cmd("netsh", &["winsock", "reset"]) {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "repair_windows_image" => match run_cmd("DISM", &["/Online", "/Cleanup-Image", "/RestoreHealth"]) {
            Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        "execute_admin_script" => match require_str_arg(args, "path") {
            Ok(script_path) => {
                let cmd = format!("powershell -NoProfile -ExecutionPolicy Bypass -File '{}'", script_path.replace('\'', "''"));
                match run_powershell(&cmd) {
                    Ok(out) => ToolResult { tool_name: name.into(), success: true, output: out, error: None },
                    Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
                }
            }
            Err(e) => ToolResult { tool_name: name.into(), success: false, output: String::new(), error: Some(e) },
        },
        _ => ToolResult {
            tool_name: name.into(),
            success: false,
            output: String::new(),
            error: Some(format!("Tool '{}' no implementada en catalog_tools", name)),
        },
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolPolicy {
    pub name: String,
    pub category: String,
    pub min_role: rbac::UserRole,
    pub sensitive: bool,
    pub megaboss_required: bool,
}

fn infer_category(tool_name: &str) -> String {
    if tool_name.contains("network")
        || tool_name.contains("dns")
        || tool_name.contains("ip")
        || tool_name.contains("wifi")
        || tool_name.contains("gateway")
        || tool_name.contains("ping")
        || tool_name.contains("traceroute")
    {
        "network".to_string()
    } else if tool_name.contains("process") {
        "process".to_string()
    } else if tool_name.contains("service") {
        "services".to_string()
    } else if tool_name.contains("disk")
        || tool_name.contains("memory")
        || tool_name.contains("cpu")
    {
        "telemetry".to_string()
    } else if tool_name.contains("file") || tool_name.contains("directory") {
        "filesystem".to_string()
    } else if tool_name.contains("registry") || tool_name.contains("security") {
        "security".to_string()
    } else {
        "system".to_string()
    }
}

fn infer_min_role(tool_name: &str) -> rbac::UserRole {
    if rbac::VIEWER_TOOLS.contains(&tool_name) {
        rbac::UserRole::Viewer
    } else if rbac::POWER_USER_TOOLS.contains(&tool_name) {
        rbac::UserRole::PowerUser
    } else {
        rbac::UserRole::Owner
    }
}

pub fn get_tool_policies() -> Vec<ToolPolicy> {
    let mut defs = crate::tools::ToolEngine::get_tool_definitions();
    defs.sort_by(|a, b| a.name.cmp(&b.name));
    defs.dedup_by(|a, b| a.name == b.name);

    defs.into_iter()
        .map(|d| {
            let megaboss_required = rbac::is_owner_only_tool(&d.name);
            let min_role = infer_min_role(&d.name);
            let sensitive = min_role != rbac::UserRole::Viewer;
            ToolPolicy {
                name: d.name.clone(),
                category: infer_category(&d.name),
                min_role,
                sensitive,
                megaboss_required,
            }
        })
        .collect()
}

pub fn get_tool_policy(tool_name: &str) -> Option<ToolPolicy> {
    get_tool_policies()
        .into_iter()
        .find(|p| p.name == tool_name)
}
