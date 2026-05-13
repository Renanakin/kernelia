use crate::tools::ToolResult;
use serde::Deserialize;
use std::os::windows::process::CommandExt;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Deserialize)]
struct DriverIssueRaw {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "PNPDeviceID")]
    pnp_device_id: Option<String>,
    #[serde(rename = "ConfigManagerErrorCode")]
    error_code: Option<u32>,
    #[serde(rename = "Manufacturer")]
    manufacturer: Option<String>,
}

fn code_to_text(code: u32) -> &'static str {
    match code {
        1 => "No configurado correctamente",
        3 => "Controlador dañado o ausente",
        10 => "No se puede iniciar (Code 10)",
        12 => "Recursos insuficientes/conflicto",
        14 => "Requiere reinicio",
        18 => "Reinstalar controladores",
        22 => "Dispositivo deshabilitado",
        24 => "No presente/instalacion incompleta",
        28 => "No hay controlador instalado",
        31 => "Windows no puede cargar el controlador",
        32 => "Servicio del controlador deshabilitado",
        37 => "Fallo al inicializar controlador",
        39 => "Controlador corrupto o faltante",
        43 => "Dispositivo detenido por error",
        _ => "Error de controlador/dispositivo",
    }
}

fn query_driver_issues() -> Result<Vec<DriverIssueRaw>, String> {
    let script = r#"
$ErrorActionPreference = 'Stop'
$items = Get-CimInstance Win32_PnPEntity | Where-Object { $_.ConfigManagerErrorCode -ne 0 } |
  Select-Object Name, PNPDeviceID, ConfigManagerErrorCode, Manufacturer
if ($null -eq $items) { '[]' } else { $items | ConvertTo-Json -Depth 4 -Compress }
"#;

    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() || text == "null" {
        return Ok(Vec::new());
    }

    if text.starts_with('[') {
        serde_json::from_str::<Vec<DriverIssueRaw>>(&text).map_err(|e| e.to_string())
    } else {
        let one = serde_json::from_str::<DriverIssueRaw>(&text).map_err(|e| e.to_string())?;
        Ok(vec![one])
    }
}

pub fn list_driver_issues() -> ToolResult {
    match query_driver_issues() {
        Ok(issues) => {
            if issues.is_empty() {
                return ToolResult {
                    tool_name: "list_driver_issues".into(),
                    success: true,
                    output: "No se detectaron controladores con problemas (ConfigManagerErrorCode != 0).".into(),
                    error: None,
                };
            }

            let mut out = String::from("Controladores/dispositivos con problemas detectados:\n\n");
            for (idx, issue) in issues.iter().enumerate() {
                let code = issue.error_code.unwrap_or(0);
                let name = issue.name.clone().unwrap_or_else(|| "Desconocido".into());
                let pnp = issue.pnp_device_id.clone().unwrap_or_else(|| "N/A".into());
                let maker = issue.manufacturer.clone().unwrap_or_else(|| "N/A".into());
                out.push_str(&format!(
                    "{}. {}\n   - Codigo: {} ({})\n   - Fabricante: {}\n   - PNPDeviceID: {}\n\n",
                    idx + 1,
                    name,
                    code,
                    code_to_text(code),
                    maker,
                    pnp
                ));
            }

            out.push_str("Opciones disponibles:\n");
            out.push_str("- update_problem_drivers: intenta re-detectar hardware y actualizar desde Windows/driver store.\n");
            out.push_str(
                "- search_missing_driver: abre Windows Update (controladores opcionales).\n",
            );

            ToolResult {
                tool_name: "list_driver_issues".into(),
                success: true,
                output: out,
                error: None,
            }
        }
        Err(e) => ToolResult {
            tool_name: "list_driver_issues".into(),
            success: false,
            output: String::new(),
            error: Some(format!(
                "No se pudo consultar el estado de controladores: {}",
                e
            )),
        },
    }
}

pub fn update_problem_drivers() -> ToolResult {
    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-Command",
            "Start-Process 'ms-settings:windowsupdate-optionalupdates'; 'OK'",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    match output {
        Ok(o) => ToolResult {
            tool_name: "update_problem_drivers".into(),
            success: o.status.success(),
            output: if o.status.success() {
                "Por seguridad, no se ejecuta re-detección forzada de hardware. Se abrió Windows Update (controladores opcionales) para actualizar sin reiniciar USB automáticamente.".into()
            } else {
                String::new()
            },
            error: if o.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&o.stderr).to_string())
            },
        },
        Err(e) => ToolResult {
            tool_name: "update_problem_drivers".into(),
            success: false,
            output: String::new(),
            error: Some(e.to_string()),
        },
    }
}

pub fn search_missing_driver() -> ToolResult {
    let script = r#"
Start-Process "ms-settings:windowsupdate-optionalupdates"
'OK'
"#;
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", script])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    match output {
        Ok(o) => ToolResult {
            tool_name: "search_missing_driver".into(),
            success: o.status.success(),
            output: if o.status.success() {
                "Se abrió Windows Update > Actualizaciones opcionales para buscar/instalar controladores faltantes.".into()
            } else {
                String::new()
            },
            error: if o.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&o.stderr).to_string())
            },
        },
        Err(e) => ToolResult {
            tool_name: "search_missing_driver".into(),
            success: false,
            output: String::new(),
            error: Some(e.to_string()),
        },
    }
}
