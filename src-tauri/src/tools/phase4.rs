use crate::config::AppSettings;
use crate::tools::{phase2, phase3, scheduler, ToolResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProactiveAlert {
    id: String,
    created_at: DateTime<Utc>,
    severity: String,
    source: String,
    title: String,
    details: String,
    suggested_actions: Vec<String>,
    auto_fix_applied: bool,
}

fn phase4_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("nexus-lite")
        .join("phase4")
}

fn ensure_phase4_dir() -> Result<(), String> {
    fs::create_dir_all(phase4_dir()).map_err(|e| e.to_string())
}

fn alerts_path() -> PathBuf {
    phase4_dir().join("proactive_alerts.jsonl")
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

fn read_alerts() -> Vec<ProactiveAlert> {
    let path = alerts_path();
    if !path.exists() {
        return vec![];
    }

    let content = fs::read_to_string(path).unwrap_or_default();
    content
        .lines()
        .filter_map(|line| serde_json::from_str::<ProactiveAlert>(line).ok())
        .collect()
}

fn build_alert_from_health(
    health: &serde_json::Value,
    multiagent: &serde_json::Value,
    auto_fix_applied: bool,
) -> ProactiveAlert {
    let health_score = health["health_score"].as_i64().unwrap_or(100);
    let risk = health["risk"].as_str().unwrap_or("bajo");
    let severity = if risk == "alto" || health_score < 45 {
        "high"
    } else if risk == "medio" || health_score < 70 {
        "medium"
    } else {
        "low"
    };

    let suggested_actions = multiagent["recommended_actions"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let title = match severity {
        "high" => "Riesgo operativo alto detectado",
        "medium" => "Riesgo operativo medio detectado",
        _ => "Estado estable con monitoreo proactivo",
    }
    .to_string();

    ProactiveAlert {
        id: format!("ALERT-{}", uuid::Uuid::new_v4().simple()),
        created_at: Utc::now(),
        severity: severity.to_string(),
        source: "phase4-proactive-maintenance".to_string(),
        title,
        details: format!(
            "health_score={} risk={} actions={}",
            health_score,
            risk,
            suggested_actions.len()
        ),
        suggested_actions,
        auto_fix_applied,
    }
}

fn choose_model(
    task_kind: &str,
    privacy_level: &str,
    urgency: &str,
) -> (
    Option<crate::config::settings::ModelConfig>,
    Vec<String>,
    String,
) {
    let settings = AppSettings::load();
    let available = settings
        .models
        .into_iter()
        .filter(|m| m.is_local || m.api_key.is_some())
        .collect::<Vec<_>>();

    let ids = available.iter().map(|m| m.id.clone()).collect::<Vec<_>>();

    let rationale = if privacy_level.eq_ignore_ascii_case("strict") {
        "Privacidad estricta: se prioriza modelo local.".to_string()
    } else if urgency.eq_ignore_ascii_case("high") {
        "Urgencia alta: se prioriza proveedor de baja latencia o local disponible.".to_string()
    } else if task_kind.eq_ignore_ascii_case("long_context") {
        "Tarea de contexto largo: se prioriza modelo con mayor max_tokens.".to_string()
    } else {
        "Balance general entre privacidad, latencia y capacidad.".to_string()
    };

    let selected = if privacy_level.eq_ignore_ascii_case("strict") {
        available.iter().find(|m| m.is_local).cloned()
    } else if urgency.eq_ignore_ascii_case("high") {
        available
            .iter()
            .find(|m| m.provider.contains("groq") || m.is_local)
            .cloned()
            .or_else(|| available.first().cloned())
    } else if task_kind.eq_ignore_ascii_case("long_context") {
        available.iter().max_by_key(|m| m.max_tokens).cloned()
    } else {
        available
            .iter()
            .find(|m| m.supports_function_calling)
            .cloned()
            .or_else(|| available.first().cloned())
    };

    (selected, ids, rationale)
}

fn ensure_scheduled_task(name: &str, command: &str, interval_hours: u64) -> serde_json::Value {
    let listed = scheduler::list_scheduled_tasks();
    let tasks =
        serde_json::from_str::<Vec<scheduler::ScheduledTask>>(&listed.output).unwrap_or_default();

    if let Some(existing) = tasks
        .iter()
        .find(|t| t.name == name && t.command == command && t.enabled)
    {
        return json!({
            "created": false,
            "task_id": existing.id,
            "name": existing.name,
            "command": existing.command,
            "interval_hours": existing.interval_hours,
            "next_run": existing.next_run,
            "note": "Tarea existente reutilizada (idempotencia)"
        });
    }

    let created = scheduler::schedule_maintenance(name, interval_hours, command);
    json!({
        "created": created.success,
        "name": name,
        "command": command,
        "interval_hours": interval_hours,
        "raw_output": created.output,
        "error": created.error
    })
}

pub async fn run_proactive_maintenance(
    app: &tauri::AppHandle,
    role: crate::tools::rbac::UserRole,
    execute_actions: bool,
) -> ToolResult {
    if let Err(e) = ensure_phase4_dir() {
        return ToolResult {
            tool_name: "run_proactive_maintenance".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    let health_raw = phase2::health_overview();
    let health =
        serde_json::from_str::<serde_json::Value>(&health_raw.output).unwrap_or_else(|_| json!({}));

    let multiagent_raw = phase3::run_multiagent_diagnosis(&json!({
        "create_ticket_on_critical": true
    }));
    let multiagent = serde_json::from_str::<serde_json::Value>(&multiagent_raw.output)
        .unwrap_or_else(|_| json!({}));

    let risk = health["risk"].as_str().unwrap_or("bajo");
    let mut executed_actions = Vec::new();

    if execute_actions && (risk == "alto" || risk == "medio") {
        let automation = phase2::run_automation_cycle(app, role, true).await;
        executed_actions.push(json!({
            "tool": "run_automation_cycle",
            "success": automation.success,
            "error": automation.error
        }));

        let snapshot = phase3::create_rollback_snapshot(&json!({
            "reason": "Snapshot automático antes de remediación proactiva"
        }));
        executed_actions.push(json!({
            "tool": "create_rollback_snapshot",
            "success": snapshot.success,
            "error": snapshot.error
        }));
    }

    let alert = build_alert_from_health(
        &health,
        &multiagent,
        execute_actions && !executed_actions.is_empty(),
    );
    let _ = append_jsonl(&alerts_path(), &alert);

    ToolResult {
        tool_name: "run_proactive_maintenance".into(),
        success: true,
        output: json!({
            "health": health,
            "multiagent": multiagent,
            "execute_actions": execute_actions,
            "executed_actions": executed_actions,
            "alert": alert
        })
        .to_string(),
        error: None,
    }
}

pub fn list_proactive_alerts(args: &serde_json::Value) -> ToolResult {
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(30) as usize;

    let mut rows = read_alerts();
    rows.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    ToolResult {
        tool_name: "list_proactive_alerts".into(),
        success: true,
        output: serde_json::to_string(&rows.into_iter().take(limit).collect::<Vec<_>>())
            .unwrap_or_else(|_| "[]".to_string()),
        error: None,
    }
}

pub fn schedule_proactive_automation(args: &serde_json::Value) -> ToolResult {
    let interval_hours = args
        .get("interval_hours")
        .and_then(|v| v.as_u64())
        .unwrap_or(12)
        .clamp(1, 24 * 30);

    let maintenance = ensure_scheduled_task(
        "Mantenimiento Proactivo",
        "run_proactive_maintenance",
        interval_hours,
    );

    let smoke_interval = (interval_hours * 14).clamp(24, 24 * 30);
    let smoke = ensure_scheduled_task(
        "Validacion Fase4 (smoke)",
        "run_phase4_smoke",
        smoke_interval,
    );

    let success = maintenance
        .get("error")
        .map(|v| v.is_null())
        .unwrap_or(true)
        && smoke.get("error").map(|v| v.is_null()).unwrap_or(true);

    ToolResult {
        tool_name: "schedule_proactive_automation".into(),
        success,
        output: json!({
            "interval_hours": interval_hours,
            "maintenance_task": {
                "success": maintenance.get("error").map(|v| v.is_null()).unwrap_or(true),
                "details": maintenance
            },
            "smoke_task": {
                "success": smoke.get("error").map(|v| v.is_null()).unwrap_or(true),
                "details": smoke
            }
        })
        .to_string(),
        error: if success {
            None
        } else {
            Some("No se pudieron programar todas las tareas proactivas.".to_string())
        },
    }
}

pub fn recommend_model_route(args: &serde_json::Value) -> ToolResult {
    let task_kind = args
        .get("task_kind")
        .and_then(|v| v.as_str())
        .unwrap_or("general");
    let privacy_level = args
        .get("privacy_level")
        .and_then(|v| v.as_str())
        .unwrap_or("balanced");
    let urgency = args
        .get("urgency")
        .and_then(|v| v.as_str())
        .unwrap_or("normal");

    let (selected, candidates, rationale) = choose_model(task_kind, privacy_level, urgency);
    let route_status = if selected.is_some() {
        "ready"
    } else {
        "no_candidates"
    };

    ToolResult {
        tool_name: "recommend_model_route".into(),
        success: true,
        output: json!({
            "task_kind": task_kind,
            "privacy_level": privacy_level,
            "urgency": urgency,
            "selected_model": selected.as_ref().map(|m| json!({
                "id": m.id,
                "name": m.name,
                "provider": m.provider,
                "is_local": m.is_local,
                "max_tokens": m.max_tokens
            })),
            "candidates": candidates,
            "rationale": rationale,
            "route_status": route_status
        })
        .to_string(),
        error: None,
    }
}

pub fn apply_recommended_model_route(args: &serde_json::Value) -> ToolResult {
    let task_kind = args
        .get("task_kind")
        .and_then(|v| v.as_str())
        .unwrap_or("general");
    let privacy_level = args
        .get("privacy_level")
        .and_then(|v| v.as_str())
        .unwrap_or("balanced");
    let urgency = args
        .get("urgency")
        .and_then(|v| v.as_str())
        .unwrap_or("normal");

    let (selected, _, rationale) = choose_model(task_kind, privacy_level, urgency);
    let Some(selected) = selected else {
        return ToolResult {
            tool_name: "apply_recommended_model_route".into(),
            success: false,
            output: String::new(),
            error: Some(
                "No hay modelos disponibles con credenciales/configuración válida.".to_string(),
            ),
        };
    };

    let mut settings = AppSettings::load();
    settings.selected_model = selected.id.clone();
    if let Err(e) = settings.save() {
        return ToolResult {
            tool_name: "apply_recommended_model_route".into(),
            success: false,
            output: String::new(),
            error: Some(format!("No se pudo guardar settings: {}", e)),
        };
    }

    ToolResult {
        tool_name: "apply_recommended_model_route".into(),
        success: true,
        output: json!({
            "selected_model": {
                "id": selected.id,
                "name": selected.name,
                "provider": selected.provider
            },
            "rationale": rationale,
            "note": "El modelo queda persistido en settings. Si el chat estaba abierto, el cambio aplica al siguiente ciclo de configuración."
        })
        .to_string(),
        error: None,
    }
}

pub async fn run_phase4_smoke(
    app: &tauri::AppHandle,
    role: crate::tools::rbac::UserRole,
) -> ToolResult {
    let mut steps = Vec::new();

    let proactive_plan = run_proactive_maintenance(app, role, false).await;
    steps.push(json!({
        "step": "run_proactive_maintenance (plan)",
        "success": proactive_plan.success,
        "error": proactive_plan.error,
        "output": proactive_plan.output
    }));

    let scheduler = schedule_proactive_automation(&json!({
        "interval_hours": 12
    }));
    steps.push(json!({
        "step": "schedule_proactive_automation",
        "success": scheduler.success,
        "error": scheduler.error,
        "output": scheduler.output
    }));

    let route = recommend_model_route(&json!({
        "task_kind": "tool_heavy",
        "privacy_level": "balanced",
        "urgency": "high"
    }));
    steps.push(json!({
        "step": "recommend_model_route",
        "success": route.success,
        "error": route.error,
        "output": route.output
    }));

    let apply = apply_recommended_model_route(&json!({
        "task_kind": "tool_heavy",
        "privacy_level": "balanced",
        "urgency": "high"
    }));
    steps.push(json!({
        "step": "apply_recommended_model_route",
        "success": apply.success,
        "error": apply.error,
        "output": apply.output
    }));

    let alerts = list_proactive_alerts(&json!({
        "limit": 5
    }));
    steps.push(json!({
        "step": "list_proactive_alerts",
        "success": alerts.success,
        "error": alerts.error,
        "output": alerts.output
    }));

    let ok = steps
        .iter()
        .all(|s| s.get("success").and_then(|v| v.as_bool()).unwrap_or(false));

    ToolResult {
        tool_name: "run_phase4_smoke".to_string(),
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
            Some("Uno o más pasos del smoke E2E de Fase 4 fallaron.".to_string())
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_route_returns_rationale() {
        let result = recommend_model_route(&json!({
            "task_kind": "long_context",
            "privacy_level": "balanced",
            "urgency": "normal"
        }));
        assert!(result.success, "recommend_model_route should succeed");
        let payload = serde_json::from_str::<serde_json::Value>(&result.output)
            .expect("recommend_model_route must return JSON payload");
        assert!(payload["rationale"].as_str().is_some());
    }

    #[test]
    fn proactive_scheduler_returns_json() {
        let result = schedule_proactive_automation(&json!({
            "interval_hours": 6
        }));
        let payload = serde_json::from_str::<serde_json::Value>(&result.output)
            .expect("schedule_proactive_automation must return JSON payload");
        assert!(payload["maintenance_task"]["success"].as_bool().is_some());
        assert!(payload["smoke_task"]["success"].as_bool().is_some());
    }
}
