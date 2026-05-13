use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub tool: String,
    pub action: String,
    pub success: bool,
    pub output_summary: String,
    pub error: Option<String>,
}

pub fn log_action(
    _app: &tauri::AppHandle,
    tool: &str,
    args: &serde_json::Value,
    success: bool,
    error: Option<String>,
) {
    let entry = AuditEntry {
        timestamp: Utc::now(),
        tool: tool.to_string(),
        action: args.to_string(),
        success,
        output_summary: String::new(),
        error,
    };

    if let Err(e) = save_audit_entry(entry) {
        eprintln!("Failed to save audit log: {}", e);
    }
}

fn save_audit_entry(entry: AuditEntry) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_audit_log_path()?;

    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let json = serde_json::to_string(&entry)? + "\n";
    file.write_all(json.as_bytes())?;

    Ok(())
}

pub fn get_audit_log_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Ok(override_path) = std::env::var("KERNELIA_AUDIT_LOG_PATH") {
        let p = PathBuf::from(override_path);
        if !p.as_os_str().is_empty() {
            return Ok(p);
        }
    }

    let mut path = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
    path.push("nexus-lite");
    path.push("logs");
    path.push("audit.jsonl");
    Ok(path)
}

pub fn read_audit_logs(limit: usize) -> Vec<AuditEntry> {
    let path = match get_audit_log_path() {
        Ok(p) => p,
        Err(_) => return Vec::new(),
    };

    if !path.exists() {
        return Vec::new();
    }

    let content = std::fs::read_to_string(path).unwrap_or_default();
    let mut entries: Vec<AuditEntry> = content
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    entries.reverse();
    entries.into_iter().take(limit).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::{Mutex, OnceLock};

    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn lock() -> &'static Mutex<()> {
        TEST_LOCK.get_or_init(|| Mutex::new(()))
    }

    fn test_log_path() -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push("kernelia_audit_tests");
        let _ = fs::create_dir_all(&p);
        p.push("audit_test.jsonl");
        p
    }

    #[test]
    fn writes_and_reads_reverse_order() {
        let _guard = lock().lock().expect("lock poisoned");
        let path = test_log_path();
        let _ = fs::remove_file(&path);
        std::env::set_var("KERNELIA_AUDIT_LOG_PATH", &path);

        let e1 = AuditEntry {
            timestamp: Utc::now(),
            tool: "tool_a".into(),
            action: "{\"x\":1}".into(),
            success: true,
            output_summary: String::new(),
            error: None,
        };
        let e2 = AuditEntry {
            timestamp: Utc::now(),
            tool: "tool_b".into(),
            action: "{\"y\":2}".into(),
            success: false,
            output_summary: String::new(),
            error: Some("boom".into()),
        };

        save_audit_entry(e1).expect("save e1");
        save_audit_entry(e2).expect("save e2");

        let logs = read_audit_logs(10);
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].tool, "tool_b");
        assert_eq!(logs[1].tool, "tool_a");
    }

    #[test]
    fn respects_limit() {
        let _guard = lock().lock().expect("lock poisoned");
        let path = test_log_path();
        let _ = fs::remove_file(&path);
        std::env::set_var("KERNELIA_AUDIT_LOG_PATH", &path);

        for i in 0..5 {
            let entry = AuditEntry {
                timestamp: Utc::now(),
                tool: format!("tool_{i}"),
                action: "{}".into(),
                success: true,
                output_summary: String::new(),
                error: None,
            };
            save_audit_entry(entry).expect("save entry");
        }

        let logs = read_audit_logs(3);
        assert_eq!(logs.len(), 3);
        assert_eq!(logs[0].tool, "tool_4");
        assert_eq!(logs[2].tool, "tool_2");
    }
}
