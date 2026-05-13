use super::{security, ToolResult};
use std::process::Stdio;
use tokio::process::Command as TokioCommand;
use tokio::time::{timeout, Duration};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub async fn run_command(command: &str, working_dir: Option<&str>) -> ToolResult {
    log::info!("Executing command: {}", command);

    if let Err(e) = security::validate_command(command) {
        return ToolResult {
            tool_name: "secure_terminal".to_string(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = TokioCommand::new("powershell.exe");
        c.args(["-NoProfile", "-Command", command]);
        #[cfg(windows)]
        c.creation_flags(CREATE_NO_WINDOW);
        c
    } else {
        let mut c = TokioCommand::new("sh");
        c.args(["-c", command]);
        c
    };

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    match timeout(Duration::from_secs(30), cmd.output()).await {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let max_len = 10_240;
            let stdout = if stdout.len() > max_len {
                format!(
                    "{}...\n[Output truncado: {} bytes totales]",
                    &stdout[..max_len],
                    stdout.len()
                )
            } else {
                stdout
            };

            ToolResult {
                tool_name: "secure_terminal".to_string(),
                success: output.status.success(),
                output: stdout,
                error: if stderr.is_empty() {
                    None
                } else {
                    Some(stderr)
                },
            }
        }
        Ok(Err(e)) => ToolResult {
            tool_name: "secure_terminal".to_string(),
            success: false,
            output: String::new(),
            error: Some(format!("Error al iniciar la ejecucion: {}", e)),
        },
        Err(_) => ToolResult {
            tool_name: "secure_terminal".to_string(),
            success: false,
            output: String::new(),
            error: Some("TIMEOUT: El comando tardo mas de 30 segundos.".to_string()),
        },
    }
}
