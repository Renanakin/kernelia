use super::ToolResult;
use std::process::Command;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Ejecuta un cmdlet de PowerShell de forma segura y retorna ToolResult
pub fn run_powershell_cmdlet(tool_name: &str, cmdlet: &str) -> ToolResult {
    let script = format!("{} | ConvertTo-Json -Compress", cmdlet);

    let mut cmd = Command::new("powershell.exe");
    cmd.args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &script]);

    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

            if output.status.success() {
                ToolResult {
                    tool_name: tool_name.to_string(),
                    success: true,
                    output: if stdout.is_empty() { "[]".to_string() } else { stdout },
                    error: None,
                }
            } else {
                ToolResult {
                    tool_name: tool_name.to_string(),
                    success: false,
                    output: String::new(),
                    error: Some(format!("Error al ejecutar PowerShell (exit code {:?}): {}", output.status.code(), stderr)),
                }
            }
        }
        Err(e) => ToolResult {
            tool_name: tool_name.to_string(),
            success: false,
            output: String::new(),
            error: Some(format!("No se pudo invocar powershell.exe: {}", e)),
        },
    }
}

/// Diagnóstico de Red (NetworkAgent): Get-NetIPConfiguration
pub fn get_net_ip_config_ps() -> ToolResult {
    run_powershell_cmdlet("get_net_ip_config", "Get-NetIPConfiguration | Select-Object InterfaceAlias, IPv4Address, IPv4DefaultGateway, DNSServer")
}

/// Diagnóstico de Adaptadores de Red (NetworkAgent): Get-NetAdapter
pub fn get_net_adapter_ps() -> ToolResult {
    run_powershell_cmdlet("get_net_adapter", "Get-NetAdapter | Select-Object Name, InterfaceDescription, Status, LinkSpeed, MacAddress")
}

/// Diagnóstico de Conexiones TCP (NetworkAgent): Get-NetTCPConnection
pub fn get_net_tcp_connection_ps() -> ToolResult {
    run_powershell_cmdlet("get_net_tcp_connection", "Get-NetTCPConnection -State Established,Listen | Select-Object LocalAddress, LocalPort, RemoteAddress, RemotePort, State, OwningProcess | Select-Object -First 25")
}

/// Diagnóstico de Información del Equipo (SystemAgent): Get-ComputerInfo
pub fn get_computer_info_ps() -> ToolResult {
    run_powershell_cmdlet("get_computer_info", "Get-ComputerInfo | Select-Object CsName, OsName, OsVersion, OsArchitecture, WindowsBiosVersion, CsModel, CsManufacturer")
}

/// Diagnóstico de Parches Instalados (SystemAgent): Get-HotFix
pub fn get_hotfix_ps() -> ToolResult {
    run_powershell_cmdlet("get_hotfix", "Get-HotFix | Select-Object Source, Description, HotFixID, InstalledOn | Select-Object -First 20")
}

/// Diagnóstico de Top Procesos por CPU (ProcessAgent): Get-Process Top CPU
pub fn get_top_cpu_processes_ps() -> ToolResult {
    run_powershell_cmdlet("get_top_cpu_processes", "Get-Process | Sort-Object CPU -Descending | Select-Object -First 10 Id, ProcessName, CPU, @{Name='WorkingSetMB';Expression={[math]::Round($_.WorkingSet64/1MB,2)}}")
}

/// Diagnóstico de Top Procesos por RAM (ProcessAgent): Get-Process Top RAM
pub fn get_top_ram_processes_ps() -> ToolResult {
    run_powershell_cmdlet("get_top_ram_processes", "Get-Process | Sort-Object WorkingSet64 -Descending | Select-Object -First 10 Id, ProcessName, CPU, @{Name='WorkingSetMB';Expression={[math]::Round($_.WorkingSet64/1MB,2)}}")
}

/// Diagnóstico de Métricas de Rendimiento (PerformanceAgent): Get-Counter
pub fn get_performance_counters_ps() -> ToolResult {
    run_powershell_cmdlet("get_performance_counters", "(Get-Counter '\\Processor(_Total)\\% Processor Time', '\\Memory\\Available MBytes').CounterSamples | Select-Object Path, CookedValue")
}

// --- FASE 2: DIAGNÓSTICO AVANZADO E INSPECCIÓN DE ERRORES (R1) ---

/// Detección de Dispositivos PnP con Error o Código 43 (DriversAgent): Get-PnpDevice Errors
pub fn get_pnp_device_errors_ps() -> ToolResult {
    run_powershell_cmdlet("get_pnp_device_errors", "Get-PnpDevice -Status Error,Unknown | Select-Object InstanceId, FriendlyName, Class, Status")
}

/// Inspección de Estado de Windows Defender (SecurityAgent): Get-MpComputerStatus
pub fn get_defender_status_ps() -> ToolResult {
    run_powershell_cmdlet("get_defender_status", "Get-MpComputerStatus | Select-Object AntivirusEnabled, RealTimeProtectionEnabled, AntivirusSignatureAge, IsTamperProtected")
}

/// Auditoría de Eventos Críticos de Sistema (SecurityAgent): Get-WinEvent Level 1,2
pub fn audit_system_errors_ps() -> ToolResult {
    run_powershell_cmdlet("audit_system_errors", "Get-WinEvent -FilterHashtable @{LogName='System'; Level=1,2} -MaxEvents 15 | Select-Object TimeCreated, ProviderName, Id, Message")
}

/// Diagnóstico de Salud de Discos Físicos (FilesystemAgent): Get-PhysicalDisk SMART
pub fn get_physical_disk_health_ps() -> ToolResult {
    run_powershell_cmdlet("get_physical_disk_health", "Get-PhysicalDisk | Select-Object DeviceId, FriendlyName, OperationalStatus, HealthStatus, MediaType, Size")
}

/// Detección de Servicios Automáticos Detenidos (ServicesAgent): Get-Service Failed
pub fn get_failed_services_ps() -> ToolResult {
    run_powershell_cmdlet("get_failed_services", "Get-Service | Where-Object {$_.StartType -eq 'Automatic' -and $_.Status -ne 'Running'} | Select-Object Name, DisplayName, Status")
}

/// Escaneo de Integridad de Volumen sin desmontar (FilesystemAgent): Repair-Volume -Scan
pub fn repair_volume_scan_ps(drive_letter: &str) -> ToolResult {
    let cmd = format!("Repair-Volume -DriveLetter {} -Scan", drive_letter);
    run_powershell_cmdlet("repair_volume_scan", &cmd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn executes_powershell_cmdlet_successfully() {
        let res = run_powershell_cmdlet("test_get_date", "Get-Date");
        assert!(res.success, "PowerShell Get-Date falló: {:?}", res.error);
    }
}
