use super::ToolResult;
use serde_json::json;
use sysinfo::System;

/// Lista los procesos en formato JSON (para el TelemetryPanel)
pub fn list_processes_json(sort_by: &str, limit: usize) -> String {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut procs: Vec<serde_json::Value> = sys
        .processes()
        .values()
        .map(|p| {
            json!({
                "name": p.name().to_string_lossy().to_string(),
                "pid": p.pid().as_u32(),
                "cpu_usage": p.cpu_usage(),
                "memory_usage": p.memory(),
            })
        })
        .collect();

    match sort_by {
        "cpu" => procs.sort_by(|a, b| {
            b["cpu_usage"]
                .as_f64()
                .unwrap_or(0.0)
                .partial_cmp(&a["cpu_usage"].as_f64().unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        "name" => procs.sort_by(|a, b| {
            a["name"]
                .as_str()
                .unwrap_or("")
                .to_lowercase()
                .cmp(&b["name"].as_str().unwrap_or("").to_lowercase())
        }),
        _ => procs.sort_by(|a, b| {
            b["memory_usage"]
                .as_u64()
                .unwrap_or(0)
                .cmp(&a["memory_usage"].as_u64().unwrap_or(0))
        }), // default: memory
    }

    procs.truncate(limit);
    serde_json::to_string(&procs).unwrap_or_else(|_| "[]".into())
}

/// Lista los procesos activos ordenados por uso de recursos (formato texto para IA)
pub fn list_processes(sort_by: &str, limit: usize) -> ToolResult {
    let mut sys = System::new_all();
    sys.refresh_all();
    // Small delay + second refresh for accurate CPU readings
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_all();

    let mut procs: Vec<(String, u32, f32, u64)> = sys
        .processes()
        .values()
        .map(|p| {
            (
                p.name().to_string_lossy().to_string(),
                p.pid().as_u32(),
                p.cpu_usage(),
                p.memory(),
            )
        })
        .collect();

    match sort_by {
        "cpu" => procs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal)),
        "name" => procs.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase())),
        _ => procs.sort_by(|a, b| b.3.cmp(&a.3)), // default: memory
    }

    procs.truncate(limit);

    let mut output = format!(
        "{:<8} {:<30} {:>8} {:>10}\n",
        "PID", "Name", "CPU %", "Memory"
    );
    output.push_str(&"-".repeat(60));
    output.push('\n');

    for (name, pid, cpu, memory) in &procs {
        let mem_str = if *memory < 1024 * 1024 {
            format!("{:.0} KB", *memory as f64 / 1024.0)
        } else if *memory < 1024 * 1024 * 1024 {
            format!("{:.1} MB", *memory as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", *memory as f64 / (1024.0 * 1024.0 * 1024.0))
        };

        let name_display = if name.len() > 28 {
            format!("{}â€¦", &name[..27])
        } else {
            name.clone()
        };

        output.push_str(&format!(
            "{:<8} {:<30} {:>7.1}% {:>10}\n",
            pid, name_display, cpu, mem_str
        ));
    }

    output.push_str(&format!("\nTotal processes: {}", sys.processes().len()));

    ToolResult {
        tool_name: "list_processes".to_string(),
        success: true,
        output,
        error: None,
    }
}

/// Termina un proceso por PID o nombre
pub fn kill_process(pid: Option<u32>, name: Option<&str>) -> ToolResult {
    let sys = System::new_all();

    if let Some(pid_val) = pid {
        let pid = sysinfo::Pid::from_u32(pid_val);
        if let Some(process) = sys.process(pid) {
            let proc_name = process.name().to_string_lossy().to_string();
            if process.kill() {
                return ToolResult {
                    tool_name: "kill_process".to_string(),
                    success: true,
                    output: format!("Process killed: {} (PID: {})", proc_name, pid_val),
                    error: None,
                };
            }

            return ToolResult {
                tool_name: "kill_process".to_string(),
                success: false,
                output: String::new(),
                error: Some(format!(
                    "Failed to kill process {} (PID: {}).",
                    proc_name, pid_val
                )),
            };
        }

        return ToolResult {
            tool_name: "kill_process".to_string(),
            success: false,
            output: String::new(),
            error: Some(format!("Process with PID {} not found", pid_val)),
        };
    }

    if let Some(proc_name) = name {
        let matching: Vec<_> = sys
            .processes()
            .values()
            .filter(|p| {
                p.name()
                    .to_string_lossy()
                    .to_lowercase()
                    .contains(&proc_name.to_lowercase())
            })
            .collect();

        if matching.is_empty() {
            return ToolResult {
                tool_name: "kill_process".to_string(),
                success: false,
                output: String::new(),
                error: Some(format!("No processes found matching '{}'", proc_name)),
            };
        }

        let mut killed = 0;
        let mut failed = 0;
        for process in &matching {
            if process.kill() {
                killed += 1;
            } else {
                failed += 1;
            }
        }

        return ToolResult {
            tool_name: "kill_process".to_string(),
            success: killed > 0,
            output: format!(
                "Killed {} of {} processes matching '{}'",
                killed,
                matching.len(),
                proc_name
            ),
            error: if failed > 0 {
                Some(format!("{} processes could not be killed", failed))
            } else {
                None
            },
        };
    }

    ToolResult {
        tool_name: "kill_process".to_string(),
        success: false,
        output: String::new(),
        error: Some("Provide either 'pid' or 'name' to kill a process".to_string()),
    }
}
