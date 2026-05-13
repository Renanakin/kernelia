use crate::tools::ToolResult;
use std::os::windows::process::CommandExt;
use std::process::Command;

pub fn list_running_services() -> ToolResult {
    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-Command",
            "Get-Service | Where-Object { $_.Status -eq 'Running' } | Select-Object Name, DisplayName, Status | ConvertTo-Json",
        ])
        .creation_flags(0x08000000)
        .output();

    match output {
        Ok(o) => ToolResult {
            tool_name: "list_running_services".into(),
            success: o.status.success(),
            output: String::from_utf8_lossy(&o.stdout).to_string(),
            error: if o.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&o.stderr).to_string())
            },
        },
        Err(e) => ToolResult {
            tool_name: "list_running_services".into(),
            success: false,
            output: String::new(),
            error: Some(e.to_string()),
        },
    }
}

pub fn get_service_info(name: &str) -> ToolResult {
    let output = Command::new("sc")
        .args(["query", name])
        .creation_flags(0x08000000)
        .output();

    match output {
        Ok(o) => ToolResult {
            tool_name: "get_service_info".into(),
            success: o.status.success(),
            output: String::from_utf8_lossy(&o.stdout).to_string(),
            error: None,
        },
        Err(e) => ToolResult {
            tool_name: "get_service_info".into(),
            success: false,
            output: String::new(),
            error: Some(e.to_string()),
        },
    }
}

pub fn restart_service(name: &str) -> ToolResult {
    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-Command",
            &format!("Restart-Service -Name '{}' -Force", name),
        ])
        .creation_flags(0x08000000)
        .output();

    match output {
        Ok(o) => ToolResult {
            tool_name: "restart_service".into(),
            success: o.status.success(),
            output: if o.status.success() {
                format!("Servicio '{}' reiniciado con exito.", name)
            } else {
                String::from_utf8_lossy(&o.stderr).to_string()
            },
            error: if o.status.success() {
                None
            } else {
                Some("Error de permisos o servicio no encontrado".into())
            },
        },
        Err(e) => ToolResult {
            tool_name: "restart_service".into(),
            success: false,
            output: String::new(),
            error: Some(e.to_string()),
        },
    }
}
