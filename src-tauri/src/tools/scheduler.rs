use crate::tools::ToolResult;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: String,
    pub name: String,
    pub command: String,
    pub next_run: DateTime<Utc>,
    pub interval_hours: u64,
    pub last_run: Option<DateTime<Utc>>,
    pub enabled: bool,
}

fn scheduler_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("nexus-lite")
        .join("scheduler")
}

pub fn schedule_maintenance(name: &str, interval_hours: u64, command: &str) -> ToolResult {
    let config_dir = scheduler_dir();
    let _ = fs::create_dir_all(&config_dir);
    let task_id = uuid::Uuid::new_v4().to_string();
    let next_run = Utc::now() + Duration::hours(interval_hours as i64);

    let task = ScheduledTask {
        id: task_id.clone(),
        name: name.to_string(),
        command: command.to_string(),
        next_run,
        interval_hours,
        last_run: None,
        enabled: true,
    };

    let file_path = config_dir.join(format!("{}.json", task_id));
    match serde_json::to_string_pretty(&task) {
        Ok(json) => match fs::write(file_path, json) {
            Ok(_) => ToolResult {
                tool_name: "schedule_maintenance".into(),
                success: true,
                output: format!(
                    "Tarea '{}' programada. ID: {}. Proxima ejecucion: {}",
                    name, task_id, next_run
                ),
                error: None,
            },
            Err(e) => ToolResult {
                tool_name: "schedule_maintenance".into(),
                success: false,
                output: String::new(),
                error: Some(format!("Error saving task: {}", e)),
            },
        },
        Err(e) => ToolResult {
            tool_name: "schedule_maintenance".into(),
            success: false,
            output: String::new(),
            error: Some(format!("Error serializing task: {}", e)),
        },
    }
}

pub fn list_scheduled_tasks() -> ToolResult {
    let config_dir = scheduler_dir();
    if !config_dir.exists() {
        return ToolResult {
            tool_name: "list_scheduled_tasks".into(),
            success: true,
            output: "[]".into(),
            error: None,
        };
    }

    let mut tasks = Vec::new();
    if let Ok(entries) = fs::read_dir(config_dir) {
        for entry in entries.flatten() {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(task) = serde_json::from_str::<ScheduledTask>(&content) {
                    tasks.push(task);
                }
            }
        }
    }

    ToolResult {
        tool_name: "list_scheduled_tasks".into(),
        success: true,
        output: serde_json::to_string(&tasks).unwrap_or_else(|_| "[]".into()),
        error: None,
    }
}

pub async fn run_pending_tasks(app: &tauri::AppHandle) {
    let config_dir = scheduler_dir();
    if !config_dir.exists() {
        return;
    }

    if let Ok(entries) = fs::read_dir(config_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(mut task) = serde_json::from_str::<ScheduledTask>(&content) {
                    if task.enabled && Utc::now() >= task.next_run {
                        let settings = crate::config::AppSettings::load();
                        let args = serde_json::json!({});
                        let _ = crate::tools::ToolEngine::execute(
                            app,
                            &task.command,
                            &args,
                            settings.user_role,
                        )
                        .await;
                        task.last_run = Some(Utc::now());
                        task.next_run = Utc::now() + Duration::hours(task.interval_hours as i64);
                        if let Ok(json) = serde_json::to_string_pretty(&task) {
                            let _ = fs::write(&path, json);
                        }
                    }
                }
            }
        }
    }
}

pub fn delete_scheduled_task(id: &str) -> ToolResult {
    let file_path = scheduler_dir().join(format!("{}.json", id));
    if file_path.exists() {
        match fs::remove_file(file_path) {
            Ok(_) => ToolResult {
                tool_name: "delete_scheduled_task".into(),
                success: true,
                output: format!("Tarea {} eliminada correctamente.", id),
                error: None,
            },
            Err(e) => ToolResult {
                tool_name: "delete_scheduled_task".into(),
                success: false,
                output: String::new(),
                error: Some(format!("Error eliminando archivo: {}", e)),
            },
        }
    } else {
        ToolResult {
            tool_name: "delete_scheduled_task".into(),
            success: false,
            output: String::new(),
            error: Some("La tarea no existe.".into()),
        }
    }
}

pub fn toggle_scheduled_task(id: &str, enabled: bool) -> ToolResult {
    let file_path = scheduler_dir().join(format!("{}.json", id));
    if let Ok(content) = fs::read_to_string(&file_path) {
        if let Ok(mut task) = serde_json::from_str::<ScheduledTask>(&content) {
            task.enabled = enabled;
            if let Ok(json) = serde_json::to_string_pretty(&task) {
                if fs::write(&file_path, json).is_ok() {
                    return ToolResult {
                        tool_name: "toggle_scheduled_task".into(),
                        success: true,
                        output: format!(
                            "Tarea {} {}",
                            id,
                            if enabled { "activada" } else { "desactivada" }
                        ),
                        error: None,
                    };
                }
            }
        }
    }

    ToolResult {
        tool_name: "toggle_scheduled_task".into(),
        success: false,
        output: String::new(),
        error: Some("No se pudo actualizar la tarea.".into()),
    }
}
