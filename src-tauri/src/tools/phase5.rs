use crate::tools::{audit, cloud, phase2, phase3, phase4, report_generator, ToolResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SupportCase {
    id: String,
    created_at: DateTime<Utc>,
    ticket_id: String,
    severity: String,
    customer: String,
    summary: String,
    status: String,
    assigned_team: String,
}

fn phase5_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("nexus-lite")
        .join("phase5")
}

fn ensure_phase5_dir() -> Result<(), String> {
    fs::create_dir_all(phase5_dir()).map_err(|e| e.to_string())
}

fn support_cases_path() -> PathBuf {
    phase5_dir().join("support_cases.jsonl")
}

fn advanced_report_path() -> PathBuf {
    phase5_dir().join("enterprise_dashboard.md")
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

fn read_support_cases(limit: usize) -> Vec<SupportCase> {
    let path = support_cases_path();
    if !path.exists() {
        return vec![];
    }

    let content = fs::read_to_string(path).unwrap_or_default();
    let mut rows: Vec<SupportCase> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<SupportCase>(line).ok())
        .collect();
    rows.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    rows.into_iter().take(limit).collect()
}

fn read_all_support_cases() -> Vec<SupportCase> {
    read_support_cases(usize::MAX)
}

pub fn create_support_case(args: &serde_json::Value) -> ToolResult {
    if let Err(e) = ensure_phase5_dir() {
        return ToolResult {
            tool_name: "create_support_case".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    let ticket_id = args
        .get("ticket_id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            format!(
                "HT-{}",
                uuid::Uuid::new_v4().simple().to_string().to_uppercase()
            )
        });
    let customer = args
        .get("customer")
        .and_then(|v| v.as_str())
        .unwrap_or("Hackteck Customer")
        .to_string();

    // Evita duplicar casos abiertos para el mismo ticket/cliente.
    if let Some(existing) = read_all_support_cases()
        .into_iter()
        .find(|c| c.ticket_id == ticket_id && c.customer == customer && c.status == "open")
    {
        return ToolResult {
            tool_name: "create_support_case".into(),
            success: true,
            output: json!({
                "reused": true,
                "case": existing
            })
            .to_string(),
            error: None,
        };
    }

    let case = SupportCase {
        id: format!(
            "CASE-{}",
            uuid::Uuid::new_v4().simple().to_string().to_uppercase()
        ),
        created_at: Utc::now(),
        ticket_id,
        severity: args
            .get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("medium")
            .to_string(),
        customer,
        summary: args
            .get("summary")
            .and_then(|v| v.as_str())
            .unwrap_or("Caso generado desde conector enterprise")
            .to_string(),
        status: "open".to_string(),
        assigned_team: args
            .get("assigned_team")
            .and_then(|v| v.as_str())
            .unwrap_or("N1-Helpdesk")
            .to_string(),
    };

    if let Err(e) = append_jsonl(&support_cases_path(), &case) {
        return ToolResult {
            tool_name: "create_support_case".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    ToolResult {
        tool_name: "create_support_case".into(),
        success: true,
        output: serde_json::to_string(&case).unwrap_or_else(|_| "{}".to_string()),
        error: None,
    }
}

pub fn list_support_cases(args: &serde_json::Value) -> ToolResult {
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
    let status_filter = args.get("status").and_then(|v| v.as_str());
    let severity_filter = args.get("severity").and_then(|v| v.as_str());
    let customer_filter = args.get("customer").and_then(|v| v.as_str());

    let rows = read_support_cases(limit)
        .into_iter()
        .filter(|c| {
            status_filter
                .map(|s| c.status.eq_ignore_ascii_case(s))
                .unwrap_or(true)
        })
        .filter(|c| {
            severity_filter
                .map(|s| c.severity.eq_ignore_ascii_case(s))
                .unwrap_or(true)
        })
        .filter(|c| {
            customer_filter
                .map(|s| c.customer.eq_ignore_ascii_case(s))
                .unwrap_or(true)
        })
        .collect::<Vec<_>>();
    ToolResult {
        tool_name: "list_support_cases".into(),
        success: true,
        output: serde_json::to_string(&rows).unwrap_or_else(|_| "[]".to_string()),
        error: None,
    }
}

pub fn get_enterprise_dashboard() -> ToolResult {
    let reports = cloud::read_cloud_reports(500);
    let cases = read_support_cases(500);
    let tickets =
        serde_json::from_str::<Vec<serde_json::Value>>(&phase2::list_incident_tickets().output)
            .unwrap_or_default();
    let alerts = serde_json::from_str::<Vec<serde_json::Value>>(
        &phase4::list_proactive_alerts(&json!({ "limit": 500 })).output,
    )
    .unwrap_or_default();

    let avg_health = if reports.is_empty() {
        0.0
    } else {
        reports.iter().map(|r| r.health_score as f64).sum::<f64>() / reports.len() as f64
    };

    let high_risk = reports.iter().filter(|r| r.risk == "alto").count();
    let open_cases = cases.iter().filter(|c| c.status == "open").count();
    let closed_cases = cases.iter().filter(|c| c.status == "closed").count();
    let high_severity_cases = cases
        .iter()
        .filter(|c| {
            c.severity.eq_ignore_ascii_case("high") || c.severity.eq_ignore_ascii_case("critical")
        })
        .count();

    ToolResult {
        tool_name: "get_enterprise_dashboard".into(),
        success: true,
        output: json!({
            "kpis": {
                "cloud_reports": reports.len(),
                "open_support_cases": open_cases,
                "incident_tickets": tickets.len(),
                "proactive_alerts": alerts.len(),
                "avg_health_score": (avg_health * 100.0).round() / 100.0,
                "high_risk_reports": high_risk,
                "closed_support_cases": closed_cases,
                "high_severity_cases": high_severity_cases
            },
            "latest": {
                "report": reports.first(),
                "support_case": cases.first(),
                "ticket": tickets.first(),
                "alert": alerts.first()
            }
        })
        .to_string(),
        error: None,
    }
}

pub fn generate_advanced_reporting() -> ToolResult {
    if let Err(e) = ensure_phase5_dir() {
        return ToolResult {
            tool_name: "generate_advanced_reporting".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    let dashboard = get_enterprise_dashboard();
    let support_report = report_generator::generate_support_report();
    let mt_overview = phase3::cloud_multi_tenant_overview();
    let audits = audit::read_audit_logs(25);

    let dashboard_json =
        serde_json::from_str::<serde_json::Value>(&dashboard.output).unwrap_or_else(|_| json!({}));

    let markdown = format!(
        "# Enterprise Dashboard Report\n\nGenerado: {}\n\n## KPIs\n\n```json\n{}\n```\n\n## Multiempresa\n\n```json\n{}\n```\n\n## Auditoría Reciente\n\n```json\n{}\n```\n\n## Soporte Técnico Base\n\n{}\n",
        Utc::now().to_rfc3339(),
        serde_json::to_string_pretty(&dashboard_json["kpis"]).unwrap_or_else(|_| "{}".to_string()),
        serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(&mt_overview.output).unwrap_or_else(|_| json!({})))
            .unwrap_or_else(|_| "{}".to_string()),
        serde_json::to_string_pretty(&audits).unwrap_or_else(|_| "[]".to_string()),
        support_report.output
    );

    match fs::write(advanced_report_path(), markdown) {
        Ok(_) => ToolResult {
            tool_name: "generate_advanced_reporting".into(),
            success: true,
            output: format!(
                "Reporte enterprise generado en: {}",
                advanced_report_path().display()
            ),
            error: None,
        },
        Err(e) => ToolResult {
            tool_name: "generate_advanced_reporting".into(),
            success: false,
            output: String::new(),
            error: Some(e.to_string()),
        },
    }
}

pub async fn run_phase5_smoke(app: &tauri::AppHandle) -> ToolResult {
    let mut steps = Vec::new();

    let upload = cloud::upload_report(app);
    steps.push(json!({
        "step": "upload_cloud_report",
        "success": upload.success,
        "error": upload.error,
        "output": upload.output
    }));

    let list_reports = cloud::list_cloud_reports();
    steps.push(json!({
        "step": "list_cloud_reports",
        "success": list_reports.success,
        "error": list_reports.error,
        "output": list_reports.output
    }));

    let ticket_id = serde_json::from_str::<serde_json::Value>(&upload.output)
        .ok()
        .and_then(|v| v["ticket_id"].as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| {
            format!(
                "HT-{}",
                uuid::Uuid::new_v4().simple().to_string().to_uppercase()
            )
        });

    let case = create_support_case(&json!({
        "ticket_id": ticket_id,
        "severity": "medium",
        "customer": "Smoke Customer",
        "summary": "Validación Fase 5",
        "assigned_team": "N1-Helpdesk"
    }));
    steps.push(json!({
        "step": "create_support_case",
        "success": case.success,
        "error": case.error,
        "output": case.output
    }));

    let dashboard = get_enterprise_dashboard();
    steps.push(json!({
        "step": "get_enterprise_dashboard",
        "success": dashboard.success,
        "error": dashboard.error,
        "output": dashboard.output
    }));

    let advanced = generate_advanced_reporting();
    steps.push(json!({
        "step": "generate_advanced_reporting",
        "success": advanced.success,
        "error": advanced.error,
        "output": advanced.output
    }));

    let ok = steps
        .iter()
        .all(|s| s.get("success").and_then(|v| v.as_bool()).unwrap_or(false));

    ToolResult {
        tool_name: "run_phase5_smoke".into(),
        success: ok,
        output: json!({
            "success": ok,
            "executed_at": Utc::now(),
            "steps": steps
        })
        .to_string(),
        error: if ok {
            None
        } else {
            Some("Uno o más pasos del smoke E2E de Fase 5 fallaron.".to_string())
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn support_case_creation_returns_json() {
        let result = create_support_case(&json!({
            "summary": "Unit Test Case",
            "customer": "Unit Test"
        }));
        assert!(result.success);
        let payload = serde_json::from_str::<serde_json::Value>(&result.output)
            .expect("create_support_case should return JSON payload");
        assert!(payload["id"].as_str().is_some());
        assert!(payload["ticket_id"].as_str().is_some());
    }

    #[test]
    fn enterprise_dashboard_returns_kpis() {
        let result = get_enterprise_dashboard();
        assert!(result.success);
        let payload = serde_json::from_str::<serde_json::Value>(&result.output)
            .expect("get_enterprise_dashboard should return JSON payload");
        assert!(payload["kpis"].is_object());
    }

    #[test]
    fn support_case_deduplicates_open_ticket() {
        let ticket_id = format!(
            "HT-{}",
            uuid::Uuid::new_v4().simple().to_string().to_uppercase()
        );
        let first = create_support_case(&json!({
            "ticket_id": ticket_id,
            "customer": "Dedup Customer",
            "summary": "Primer caso"
        }));
        assert!(first.success);

        let second = create_support_case(&json!({
            "ticket_id": ticket_id,
            "customer": "Dedup Customer",
            "summary": "Segundo intento"
        }));
        assert!(second.success);
        let payload = serde_json::from_str::<serde_json::Value>(&second.output)
            .expect("create_support_case dedup should return JSON payload");
        assert_eq!(payload["reused"].as_bool(), Some(true));
    }
}
