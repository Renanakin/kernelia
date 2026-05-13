use super::ToolResult;
use tokio::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub async fn analyze_registry() -> ToolResult {
    let ps_script = r#"
        $paths = @(
            "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run",
            "HKLM:\Software\Microsoft\Windows\CurrentVersion\Run"
        )
        $results = @()
        $skipProps = @('PSPath','PSParentPath','PSChildName','PSDrive','PSProvider')
        foreach ($path in $paths) {
            if (Test-Path $path) {
                $item = Get-ItemProperty -Path $path
                $item.PSObject.Properties | Where-Object { $skipProps -notcontains $_.Name } | ForEach-Object {
                    $name = $_.Name
                    $cmd = [string]$_.Value
                    if ($cmd -match '(?i)"?([^"]+)"?') {
                        $exe = $matches[1].Split(" ")[0].Replace('"', '')
                        if (-not (Test-Path $exe) -and $exe -notmatch '^%') {
                            $results += "Invalido: $name -> $exe (Archivo no encontrado)"
                        }
                    }
                }
            }
        }
        if ($results.Count -eq 0) { "No se detectaron entradas de inicio huerfanas." }
        else { $results -join "`n" }
    "#;

    let mut command = Command::new("powershell.exe");
    command.args(["-NoProfile", "-Command", ps_script]);
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    let output = command.output().await;

    match output {
        Ok(out) => ToolResult {
            tool_name: "analyze_registry".to_string(),
            success: true,
            output: format!(
                "Analisis del Registro finalizado.\n\n{}",
                String::from_utf8_lossy(&out.stdout).trim()
            ),
            error: None,
        },
        Err(e) => ToolResult {
            tool_name: "analyze_registry".to_string(),
            success: false,
            output: String::new(),
            error: Some(format!("Error al analizar el registro: {}", e)),
        },
    }
}

pub async fn fix_registry() -> ToolResult {
    let ps_script = r#"
        $targets = @(
            "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run",
            "HKLM:\Software\Microsoft\Windows\CurrentVersion\Run"
        )
        $skipProps = @('PSPath','PSParentPath','PSChildName','PSDrive','PSProvider')
        $removed = @()
        $errors = @()

        foreach ($path in $targets) {
            if (-not (Test-Path $path)) { continue }
            $item = Get-ItemProperty -Path $path
            $item.PSObject.Properties | Where-Object { $skipProps -notcontains $_.Name } | ForEach-Object {
                $name = $_.Name
                $cmd = [string]$_.Value
                if ($cmd -match '(?i)"?([^"]+)"?') {
                    $exe = $matches[1].Split(" ")[0].Replace('"', '')
                    if (-not (Test-Path $exe) -and $exe -notmatch '^%') {
                        try {
                            Remove-ItemProperty -Path $path -Name $name -ErrorAction Stop
                            $removed += "$path -> $name"
                        } catch {
                            $errors += "$path -> $name ($($_.Exception.Message))"
                        }
                    }
                }
            }
        }

        [PSCustomObject]@{
            removed = $removed
            errors = $errors
        } | ConvertTo-Json -Compress
    "#;

    let mut command = Command::new("powershell.exe");
    command.args(["-NoProfile", "-Command", ps_script]);
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    let output = command.output().await;

    match output {
        Ok(out) => {
            let raw = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let parsed: serde_json::Value = serde_json::from_str(&raw)
                .unwrap_or_else(|_| serde_json::json!({"removed":[],"errors":[raw]}));
            let removed = parsed
                .get("removed")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let errors = parsed
                .get("errors")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            let removed_count = removed.len();
            let error_count = errors.len();

            let mut report = format!(
                "Reparacion de registro finalizada.\nEntradas eliminadas: {}.",
                removed_count
            );
            if error_count > 0 {
                report.push_str(&format!(
                    "\nCon errores: {} (normal en claves protegidas por permisos).",
                    error_count
                ));
            }
            if removed_count == 0 && error_count == 0 {
                report.push_str("\nNo se detectaron entradas invalidas para reparar.");
            }

            ToolResult {
                tool_name: "fix_registry".to_string(),
                success: true,
                output: report,
                error: None,
            }
        }
        Err(e) => ToolResult {
            tool_name: "fix_registry".to_string(),
            success: false,
            output: String::new(),
            error: Some(format!("Error al reparar el registro: {}", e)),
        },
    }
}
