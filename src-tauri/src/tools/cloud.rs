use crate::tools::{audit, phase2, phase4, ToolResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudReportRecord {
    pub ticket_id: String,
    pub timestamp: DateTime<Utc>,
    pub device_name: String,
    pub audit_summary: String,
    pub health_score: i64,
    pub risk: String,
    pub open_incidents: usize,
    pub proactive_alerts: usize,
}

fn phase5_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("nexus-lite")
        .join("phase5")
}

fn reports_path() -> PathBuf {
    phase5_dir().join("cloud_reports.jsonl")
}

fn ensure_phase5_dir() -> Result<(), String> {
    fs::create_dir_all(phase5_dir()).map_err(|e| e.to_string())
}

fn append_jsonl<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    let line = serde_json::to_string(value).map_err(|e| e.to_string())? + "\n";
    file.write_all(line.as_bytes()).map_err(|e| e.to_string())
}

pub fn read_cloud_reports(limit: usize) -> Vec<CloudReportRecord> {
    let path = reports_path();
    if !path.exists() {
        return vec![];
    }

    let content = fs::read_to_string(path).unwrap_or_default();
    let mut rows: Vec<CloudReportRecord> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<CloudReportRecord>(line).ok())
        .collect();
    rows.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    rows.into_iter().take(limit).collect()
}

pub fn upload_report(_app: &AppHandle) -> ToolResult {
    if let Err(e) = ensure_phase5_dir() {
        return ToolResult {
            tool_name: "upload_cloud_report".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    let now = Utc::now();
    let ticket_id = format!(
        "HT-{}",
        uuid::Uuid::new_v4().simple().to_string().to_uppercase()
    );
    let device_name = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let health = phase2::health_overview();
    let health_json =
        serde_json::from_str::<serde_json::Value>(&health.output).unwrap_or_else(|_| json!({}));
    let health_score = health_json["health_score"].as_i64().unwrap_or(100);
    let risk = health_json["risk"].as_str().unwrap_or("bajo").to_string();

    let incidents = phase2::list_incident_tickets();
    let open_incidents = serde_json::from_str::<Vec<serde_json::Value>>(&incidents.output)
        .unwrap_or_default()
        .len();

    let proactive_alerts = phase4::list_proactive_alerts(&json!({ "limit": 50 }));
    let proactive_alert_count =
        serde_json::from_str::<Vec<serde_json::Value>>(&proactive_alerts.output)
            .unwrap_or_default()
            .len();

    let recent_audit = audit::read_audit_logs(5);
    let audit_summary = if recent_audit.is_empty() {
        "Sin eventos recientes de auditoría".to_string()
    } else {
        format!(
            "{} eventos recientes; última acción: {}",
            recent_audit.len(),
            recent_audit
                .first()
                .map(|e| e.tool.clone())
                .unwrap_or_else(|| "n/a".to_string())
        )
    };

    let record = CloudReportRecord {
        ticket_id: ticket_id.clone(),
        timestamp: now,
        device_name,
        audit_summary,
        health_score,
        risk,
        open_incidents,
        proactive_alerts: proactive_alert_count,
    };

    if let Err(e) = append_jsonl(&reports_path(), &record) {
        return ToolResult {
            tool_name: "upload_cloud_report".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    ToolResult {
        tool_name: "upload_cloud_report".into(),
        success: true,
        output: json!({
            "message": "Reporte sincronizado correctamente",
            "ticket_id": ticket_id,
            "timestamp": now,
            "risk": record.risk,
            "health_score": record.health_score
        })
        .to_string(),
        error: None,
    }
}

pub fn list_cloud_reports() -> ToolResult {
    let rows = read_cloud_reports(200);
    ToolResult {
        tool_name: "list_cloud_reports".into(),
        success: true,
        output: serde_json::to_string(&rows).unwrap_or_else(|_| "[]".to_string()),
        error: None,
    }
}
