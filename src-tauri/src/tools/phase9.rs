use crate::tools::{phase7, phase8, ToolResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReliabilityEvent {
    id: String,
    timestamp: DateTime<Utc>,
    severity: String,
    category: String,
    target: String,
    observed_p95_ms: f64,
    baseline_p95_ms: f64,
    success_rate: f64,
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SelfHealingRun {
    id: String,
    executed_at: DateTime<Utc>,
    mode: String,
    readiness_score: f64,
    risk_level: String,
    actions_executed: Vec<String>,
    post_sla_status: Option<String>,
    escalated_ticket_id: Option<String>,
    summary: String,
}

fn phase8_anomalies_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("nexus-lite")
        .join("phase8")
        .join("reliability_anomalies.jsonl")
}

fn phase9_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("nexus-lite")
        .join("phase9")
}

fn ensure_phase9_dir() -> Result<(), String> {
    fs::create_dir_all(phase9_dir()).map_err(|e| e.to_string())
}

fn runs_path() -> PathBuf {
    phase9_dir().join("self_healing_runs.jsonl")
}

fn plan_path() -> PathBuf {
    phase9_dir().join("self_healing_plan.md")
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

fn read_reliability_events(limit: usize) -> Vec<ReliabilityEvent> {
    let path = phase8_anomalies_path();
    if !path.exists() {
        return vec![];
    }

    let content = fs::read_to_string(path).unwrap_or_default();
    let mut rows: Vec<ReliabilityEvent> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<ReliabilityEvent>(line).ok())
        .collect();
    rows.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    rows.into_iter().take(limit).collect()
}

fn read_runs(limit: usize) -> Vec<SelfHealingRun> {
    let path = runs_path();
    if !path.exists() {
        return vec![];
    }

    let content = fs::read_to_string(path).unwrap_or_default();
    let mut rows: Vec<SelfHealingRun> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<SelfHealingRun>(line).ok())
        .collect();
    rows.sort_by(|a, b| b.executed_at.cmp(&a.executed_at));
    rows.into_iter().take(limit).collect()
}

fn compute_readiness(events: &[ReliabilityEvent], sla_met: bool) -> (f64, String) {
    let high = events.iter().filter(|e| e.severity == "high").count() as f64;
    let medium = events.iter().filter(|e| e.severity == "medium").count() as f64;
    let base = if sla_met { 100.0 } else { 80.0 };
    let score = (base - (high * 12.0) - (medium * 5.0)).clamp(0.0, 100.0);

    let risk = if score >= 85.0 {
        "low"
    } else if score >= 65.0 {
        "medium"
    } else {
        "high"
    };

    (score, risk.to_string())
}

pub fn assess_self_healing_readiness(args: &serde_json::Value) -> ToolResult {
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(120) as usize;

    let events = read_reliability_events(limit);
    let sla_res = phase8::calculate_sla_status(&json!({
        "limit": 300,
        "target_success_rate": 99.0
    }));
    let sla_json =
        serde_json::from_str::<serde_json::Value>(&sla_res.output).unwrap_or_else(|_| json!({}));
    let sla_met = sla_json["sla_met"].as_bool().unwrap_or(false);

    let (score, risk) = compute_readiness(&events, sla_met);

    ToolResult {
        tool_name: "assess_self_healing_readiness".into(),
        success: true,
        output: json!({
            "window_events": events.len(),
            "sla_met": sla_met,
            "readiness_score": (score * 100.0).round() / 100.0,
            "risk_level": risk
        })
        .to_string(),
        error: None,
    }
}

pub fn generate_self_healing_plan(args: &serde_json::Value) -> ToolResult {
    if let Err(e) = ensure_phase9_dir() {
        return ToolResult {
            tool_name: "generate_self_healing_plan".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    let readiness = assess_self_healing_readiness(args);
    let readiness_json =
        serde_json::from_str::<serde_json::Value>(&readiness.output).unwrap_or_else(|_| json!({}));
    let risk = readiness_json["risk_level"].as_str().unwrap_or("medium");

    let mut plan_steps = vec![
        "Ejecutar run_latency_probe para refrescar baseline operacional.".to_string(),
        "Ejecutar run_tool_benchmark en list_processes y run_network_diagnostic.".to_string(),
        "Recalcular detect_performance_anomalies y evaluate SLA.".to_string(),
    ];

    if risk == "high" {
        plan_steps.push("Escalar a soporte enterprise con evidencia de degradacion.".to_string());
        plan_steps.push("Aplicar ventana de mitigacion y reducir carga concurrente.".to_string());
    } else if risk == "medium" {
        plan_steps
            .push("Programar run_phase8_smoke cada 4 horas hasta estabilizacion.".to_string());
    } else {
        plan_steps.push("Mantener monitoreo preventivo con frecuencia diaria.".to_string());
    }

    let markdown = format!(
        "# Self-Healing Plan\n\nGenerado: {}\n\n## Estado de Readiness\n\n```json\n{}\n```\n\n## Plan de Accion\n\n{}\n",
        Utc::now().to_rfc3339(),
        serde_json::to_string_pretty(&readiness_json).unwrap_or_else(|_| "{}".to_string()),
        plan_steps
            .iter()
            .enumerate()
            .map(|(i, step)| format!("{}. {}", i + 1, step))
            .collect::<Vec<String>>()
            .join("\n")
    );

    match fs::write(plan_path(), markdown) {
        Ok(_) => ToolResult {
            tool_name: "generate_self_healing_plan".into(),
            success: true,
            output: json!({
                "risk_level": risk,
                "steps": plan_steps,
                "plan_path": plan_path().display().to_string()
            })
            .to_string(),
            error: None,
        },
        Err(e) => ToolResult {
            tool_name: "generate_self_healing_plan".into(),
            success: false,
            output: String::new(),
            error: Some(e.to_string()),
        },
    }
}

pub fn execute_self_healing_cycle(args: &serde_json::Value) -> ToolResult {
    if let Err(e) = ensure_phase9_dir() {
        return ToolResult {
            tool_name: "execute_self_healing_cycle".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    let execute_actions = args
        .get("execute_actions")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let readiness = assess_self_healing_readiness(&json!({ "limit": 120 }));
    let readiness_json =
        serde_json::from_str::<serde_json::Value>(&readiness.output).unwrap_or_else(|_| json!({}));
    let score = readiness_json["readiness_score"].as_f64().unwrap_or(0.0);
    let risk = readiness_json["risk_level"]
        .as_str()
        .unwrap_or("medium")
        .to_string();

    let mut executed = Vec::new();
    let mut escalated_ticket_id: Option<String> = None;
    let mut post_sla_status: Option<String> = None;
    if execute_actions {
        let probe = phase7::run_latency_probe(&json!({ "iterations": 8 }));
        if probe.success {
            executed.push("run_latency_probe".to_string());
        }

        let bench = phase7::run_tool_benchmark(&json!({
            "tool": "list_processes",
            "iterations": 4
        }));
        if bench.success {
            executed.push("run_tool_benchmark".to_string());
        }

        let anomalies = phase8::detect_performance_anomalies(&json!({
            "limit": 150,
            "p95_multiplier": 1.7,
            "min_success_rate": 95.0
        }));
        if anomalies.success {
            executed.push("detect_performance_anomalies".to_string());
        }

        let prediction = phase8::predict_operational_incidents(&json!({ "limit": 120 }));
        if prediction.success {
            executed.push("predict_operational_incidents".to_string());
        }

        let root = phase8::explain_root_cause(&json!({ "limit": 120 }));
        if root.success {
            executed.push("explain_root_cause".to_string());
        }

        let sla_after = phase8::calculate_sla_status(&json!({
            "limit": 300,
            "target_success_rate": 99.0
        }));
        if sla_after.success {
            let sla_json = serde_json::from_str::<serde_json::Value>(&sla_after.output)
                .unwrap_or_else(|_| json!({}));
            post_sla_status = sla_json["status"].as_str().map(|s| s.to_string());
            executed.push("calculate_sla_status".to_string());
        }

        let post_readiness = assess_self_healing_readiness(&json!({ "limit": 120 }));
        let post_readiness_json = serde_json::from_str::<serde_json::Value>(&post_readiness.output)
            .unwrap_or_else(|_| json!({}));
        let post_risk = post_readiness_json["risk_level"]
            .as_str()
            .unwrap_or("medium");
        if post_risk == "high" {
            let t = crate::tools::phase2::create_incident_ticket(&json!({
                "title": "Escalamiento automático Fase 9",
                "category": "self-healing",
                "severity": "high",
                "details": "Riesgo sigue alto después de ejecutar ciclo de autocuración.",
                "source": "phase9-self-healing"
            }));
            escalated_ticket_id = serde_json::from_str::<serde_json::Value>(&t.output)
                .ok()
                .and_then(|v| v["id"].as_str().map(|s| s.to_string()));
            executed.push("create_incident_ticket".to_string());
        }
    }

    let run = SelfHealingRun {
        id: format!(
            "SH-{}",
            uuid::Uuid::new_v4().simple().to_string().to_uppercase()
        ),
        executed_at: Utc::now(),
        mode: if execute_actions {
            "execute"
        } else {
            "simulate"
        }
        .to_string(),
        readiness_score: score,
        risk_level: risk.clone(),
        actions_executed: executed.clone(),
        post_sla_status: post_sla_status.clone(),
        escalated_ticket_id: escalated_ticket_id.clone(),
        summary: if execute_actions {
            "Ciclo de autocuracion ejecutado con acciones de mitigacion.".to_string()
        } else {
            "Ciclo simulado sin ejecutar acciones.".to_string()
        },
    };

    if let Err(e) = append_jsonl(&runs_path(), &run) {
        return ToolResult {
            tool_name: "execute_self_healing_cycle".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    ToolResult {
        tool_name: "execute_self_healing_cycle".into(),
        success: true,
        output: json!({
            "run": run,
            "post_sla_status": post_sla_status,
            "escalated_ticket_id": escalated_ticket_id
        })
        .to_string(),
        error: None,
    }
}

pub fn list_self_healing_runs(args: &serde_json::Value) -> ToolResult {
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
    let rows = read_runs(limit);

    ToolResult {
        tool_name: "list_self_healing_runs".into(),
        success: true,
        output: serde_json::to_string(&rows).unwrap_or_else(|_| "[]".to_string()),
        error: None,
    }
}

pub fn run_phase9_smoke() -> ToolResult {
    let mut steps = Vec::new();

    let readiness = assess_self_healing_readiness(&json!({ "limit": 120 }));
    steps.push(json!({
        "step": "assess_self_healing_readiness",
        "success": readiness.success,
        "error": readiness.error,
        "output": readiness.output
    }));

    let plan = generate_self_healing_plan(&json!({ "limit": 120 }));
    steps.push(json!({
        "step": "generate_self_healing_plan",
        "success": plan.success,
        "error": plan.error,
        "output": plan.output
    }));

    let cycle = execute_self_healing_cycle(&json!({ "execute_actions": false }));
    steps.push(json!({
        "step": "execute_self_healing_cycle",
        "success": cycle.success,
        "error": cycle.error,
        "output": cycle.output
    }));

    let runs = list_self_healing_runs(&json!({ "limit": 5 }));
    steps.push(json!({
        "step": "list_self_healing_runs",
        "success": runs.success,
        "error": runs.error,
        "output": runs.output
    }));

    let ok = steps
        .iter()
        .all(|s| s.get("success").and_then(|v| v.as_bool()).unwrap_or(false));

    ToolResult {
        tool_name: "run_phase9_smoke".into(),
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
            Some("Uno o mas pasos del smoke E2E de Fase 9 fallaron.".to_string())
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readiness_returns_structure() {
        let result = assess_self_healing_readiness(&json!({ "limit": 10 }));
        assert!(result.success);
        let payload = serde_json::from_str::<serde_json::Value>(&result.output)
            .expect("assess_self_healing_readiness should return JSON payload");
        assert!(payload["readiness_score"].is_number());
        assert!(payload["risk_level"].is_string());
    }

    #[test]
    fn self_healing_cycle_simulation_works() {
        let result = execute_self_healing_cycle(&json!({ "execute_actions": false }));
        assert!(result.success);
        let payload = serde_json::from_str::<serde_json::Value>(&result.output)
            .expect("execute_self_healing_cycle should return JSON payload");
        assert!(payload["run"]["mode"].is_string());
        assert!(payload["run"]["readiness_score"].is_number());
    }
}
