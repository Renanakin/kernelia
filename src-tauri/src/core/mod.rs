use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub topic: String,
    pub level: String,
    pub message: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedTask {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub name: String,
    pub args: serde_json::Value,
    pub status: String, // queued | running | done | failed
    pub retries: u8,
    pub max_retries: u8,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchdogState {
    pub last_heartbeat: DateTime<Utc>,
    pub app_version: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryState {
    pub recovery_mode: bool,
    pub reason: String,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Default)]
pub struct CoreState {
    pub events: VecDeque<CoreEvent>,
    pub tasks: VecDeque<QueuedTask>,
}

impl CoreState {
    pub fn new() -> Self {
        Self {
            events: VecDeque::with_capacity(500),
            tasks: VecDeque::with_capacity(500),
        }
    }
}

fn core_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("nexus-lite")
        .join("core")
}

fn ensure_core_dir() -> Result<(), String> {
    fs::create_dir_all(core_dir()).map_err(|e| e.to_string())
}

fn path_events() -> PathBuf {
    core_dir().join("events.jsonl")
}
fn path_tasks() -> PathBuf {
    core_dir().join("task_queue.json")
}
fn path_watchdog() -> PathBuf {
    core_dir().join("watchdog.json")
}
fn path_snapshots() -> PathBuf {
    core_dir().join("snapshots.jsonl")
}
fn path_recovery() -> PathBuf {
    core_dir().join("recovery_state.json")
}
fn path_dynamic_json() -> PathBuf {
    core_dir().join("dynamic_config.json")
}
fn path_dynamic_yaml() -> PathBuf {
    core_dir().join("dynamic_config.yaml")
}

pub fn emit_event(
    state: &mut CoreState,
    topic: &str,
    level: &str,
    message: &str,
    payload: serde_json::Value,
) -> Result<CoreEvent, String> {
    ensure_core_dir()?;
    let event = CoreEvent {
        id: format!("evt-{}", Utc::now().timestamp_millis()),
        timestamp: Utc::now(),
        topic: topic.to_string(),
        level: level.to_string(),
        message: message.to_string(),
        payload,
    };
    state.events.push_front(event.clone());
    while state.events.len() > 500 {
        state.events.pop_back();
    }

    let line = serde_json::to_string(&event).map_err(|e| e.to_string())? + "\n";
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path_events())
        .map_err(|e| e.to_string())?;
    file.write_all(line.as_bytes()).map_err(|e| e.to_string())?;
    Ok(event)
}

pub fn list_events(state: &CoreState, limit: usize) -> Vec<CoreEvent> {
    state.events.iter().take(limit).cloned().collect()
}

pub fn enqueue_task(
    state: &mut CoreState,
    name: &str,
    args: serde_json::Value,
) -> Result<QueuedTask, String> {
    ensure_core_dir()?;
    let task = QueuedTask {
        id: format!("tsk-{}", Utc::now().timestamp_millis()),
        timestamp: Utc::now(),
        name: name.to_string(),
        args,
        status: "queued".to_string(),
        retries: 0,
        max_retries: 3,
        last_error: None,
    };
    state.tasks.push_back(task.clone());
    persist_tasks(state)?;
    Ok(task)
}

pub fn list_tasks(state: &CoreState) -> Vec<QueuedTask> {
    state.tasks.iter().cloned().collect()
}

pub fn persist_tasks(state: &CoreState) -> Result<(), String> {
    ensure_core_dir()?;
    let tasks: Vec<QueuedTask> = state.tasks.iter().cloned().collect();
    let body = serde_json::to_string_pretty(&tasks).map_err(|e| e.to_string())?;
    fs::write(path_tasks(), body).map_err(|e| e.to_string())
}

pub fn load_tasks(state: &mut CoreState) {
    let p = path_tasks();
    if !p.exists() {
        return;
    }
    if let Ok(body) = fs::read_to_string(p) {
        if let Ok(tasks) = serde_json::from_str::<Vec<QueuedTask>>(&body) {
            state.tasks = tasks.into_iter().collect();
        }
    }
}

pub async fn process_queue_once(
    app: &tauri::AppHandle,
    state: &std::sync::Arc<std::sync::Mutex<CoreState>>,
) {
    let maybe_task = {
        let mut s = match state.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let idx = s.tasks.iter().position(|t| t.status == "queued");
        if let Some(i) = idx {
            s.tasks[i].status = "running".to_string();
            let _ = persist_tasks(&s);
            Some(s.tasks[i].clone())
        } else {
            None
        }
    };

    let task = match maybe_task {
        Some(t) => t,
        None => return,
    };

    let result = crate::tools::ToolEngine::execute(
        app,
        &task.name,
        &task.args,
        crate::tools::rbac::UserRole::Owner,
    )
    .await;

    if let Ok(mut s) = state.lock() {
        if let Some(t) = s.tasks.iter_mut().find(|t| t.id == task.id) {
            if result.success {
                t.status = "done".to_string();
                t.last_error = None;
            } else if t.retries < t.max_retries {
                t.retries += 1;
                t.status = "queued".to_string();
                t.last_error = result.error.clone();
            } else {
                t.status = "failed".to_string();
                t.last_error = result.error.clone();
            }
        }
        let _ = persist_tasks(&s);
        let _ = emit_event(
            &mut s,
            "task.queue",
            if result.success { "info" } else { "warn" },
            if result.success {
                "Tarea ejecutada"
            } else {
                "Tarea fallida/reintentada"
            },
            serde_json::json!({
                "task_id": task.id,
                "task_name": task.name,
                "success": result.success,
                "error": result.error
            }),
        );
    }
}

pub fn watchdog_heartbeat(app_version: &str) -> Result<WatchdogState, String> {
    ensure_core_dir()?;
    let w = WatchdogState {
        last_heartbeat: Utc::now(),
        app_version: app_version.to_string(),
        status: "ok".to_string(),
    };
    let body = serde_json::to_string_pretty(&w).map_err(|e| e.to_string())?;
    fs::write(path_watchdog(), body).map_err(|e| e.to_string())?;
    Ok(w)
}

pub fn watchdog_status() -> Result<WatchdogState, String> {
    let p = path_watchdog();
    if !p.exists() {
        return watchdog_heartbeat(env!("CARGO_PKG_VERSION"));
    }
    let body = fs::read_to_string(p).map_err(|e| e.to_string())?;
    serde_json::from_str::<WatchdogState>(&body).map_err(|e| e.to_string())
}

pub fn watchdog_health(seconds_threshold: i64) -> Result<serde_json::Value, String> {
    let wd = watchdog_status()?;
    let age = Utc::now()
        .signed_duration_since(wd.last_heartbeat)
        .num_seconds();
    Ok(serde_json::json!({
        "status": if age > seconds_threshold { "stale" } else { "ok" },
        "last_heartbeat": wd.last_heartbeat,
        "age_seconds": age,
        "threshold_seconds": seconds_threshold
    }))
}

pub fn create_snapshot(source: &str, data: serde_json::Value) -> Result<SystemSnapshot, String> {
    ensure_core_dir()?;
    let snap = SystemSnapshot {
        id: format!("snap-{}", Utc::now().timestamp_millis()),
        timestamp: Utc::now(),
        source: source.to_string(),
        data,
    };
    let line = serde_json::to_string(&snap).map_err(|e| e.to_string())? + "\n";
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path_snapshots())
        .map_err(|e| e.to_string())?;
    file.write_all(line.as_bytes()).map_err(|e| e.to_string())?;
    Ok(snap)
}

pub fn list_snapshots(limit: usize) -> Vec<SystemSnapshot> {
    let p = path_snapshots();
    if !p.exists() {
        return vec![];
    }
    let content = fs::read_to_string(p).unwrap_or_default();
    let mut rows: Vec<SystemSnapshot> = content
        .lines()
        .filter_map(|l| serde_json::from_str::<SystemSnapshot>(l).ok())
        .collect();
    rows.reverse();
    rows.into_iter().take(limit).collect()
}

pub fn set_recovery_mode(enabled: bool, reason: &str) -> Result<RecoveryState, String> {
    ensure_core_dir()?;
    let state = RecoveryState {
        recovery_mode: enabled,
        reason: reason.to_string(),
        detected_at: Utc::now(),
    };
    let body = serde_json::to_string_pretty(&state).map_err(|e| e.to_string())?;
    fs::write(path_recovery(), body).map_err(|e| e.to_string())?;
    Ok(state)
}

pub fn recovery_state() -> Result<RecoveryState, String> {
    let p = path_recovery();
    if !p.exists() {
        return set_recovery_mode(false, "normal");
    }
    let body = fs::read_to_string(p).map_err(|e| e.to_string())?;
    serde_json::from_str::<RecoveryState>(&body).map_err(|e| e.to_string())
}

pub fn recovery_check() -> Result<serde_json::Value, String> {
    let wd = watchdog_status()?;
    let snapshots = list_snapshots(5);
    let recovery = recovery_state()?;
    Ok(serde_json::json!({
        "watchdog": wd,
        "recovery": recovery,
        "recent_snapshots": snapshots,
        "recovery_mode_ready": true
    }))
}

pub fn save_dynamic_config_json(data: &serde_json::Value) -> Result<(), String> {
    ensure_core_dir()?;
    let body = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
    fs::write(path_dynamic_json(), body).map_err(|e| e.to_string())
}

pub fn load_dynamic_config_json() -> Result<serde_json::Value, String> {
    let p = path_dynamic_json();
    if !p.exists() {
        return Ok(serde_json::json!({}));
    }
    let body = fs::read_to_string(p).map_err(|e| e.to_string())?;
    serde_json::from_str::<serde_json::Value>(&body).map_err(|e| e.to_string())
}

pub fn save_dynamic_config_yaml(content: &str) -> Result<(), String> {
    ensure_core_dir()?;
    fs::write(path_dynamic_yaml(), content).map_err(|e| e.to_string())
}

pub fn load_dynamic_config_yaml() -> Result<String, String> {
    let p = path_dynamic_yaml();
    if !p.exists() {
        return Ok(String::new());
    }
    fs::read_to_string(p).map_err(|e| e.to_string())
}
