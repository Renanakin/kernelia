use crate::tools::{network_diagnostic, phase2, processes, security, sysinfo_tool, ToolResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KernelDiagnosisRecord {
    id: String,
    created_at: DateTime<Utc>,
    category: String,
    summary: String,
    details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KernelAutomationRun {
    id: String,
    created_at: DateTime<Utc>,
    mode: String,
    rules_triggered: Vec<String>,
    actions_planned: Vec<String>,
    actions_executed: Vec<serde_json::Value>,
    post_verification: serde_json::Value,
    escalated_ticket_id: Option<String>,
}

fn phase6_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("nexus-lite")
        .join("phase6")
}

fn ensure_phase6_dir() -> Result<(), String> {
    fs::create_dir_all(phase6_dir()).map_err(|e| e.to_string())
}

fn diagnostics_path() -> PathBuf {
    phase6_dir().join("kernel_diagnostics.jsonl")
}

fn readiness_path() -> PathBuf {
    phase6_dir().join("kernel_readiness_report.md")
}

fn automation_runs_path() -> PathBuf {
    phase6_dir().join("kernel_automation_runs.jsonl")
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

fn read_diagnostics(limit: usize) -> Vec<KernelDiagnosisRecord> {
    let path = diagnostics_path();
    if !path.exists() {
        return vec![];
    }

    let content = fs::read_to_string(path).unwrap_or_default();
    let mut rows: Vec<KernelDiagnosisRecord> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<KernelDiagnosisRecord>(line).ok())
        .collect();
    rows.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    rows.into_iter().take(limit).collect()
}

fn read_automation_runs(limit: usize) -> Vec<KernelAutomationRun> {
    let path = automation_runs_path();
    if !path.exists() {
        return vec![];
    }

    let content = fs::read_to_string(path).unwrap_or_default();
    let mut rows: Vec<KernelAutomationRun> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<KernelAutomationRun>(line).ok())
        .collect();
    rows.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    rows.into_iter().take(limit).collect()
}

fn cpu_memory_causes(cpu: f64, memory_pct: f64) -> Vec<String> {
    let mut causes = Vec::new();
    if cpu > 85.0 {
        causes.push("Alta presion de CPU por procesos en primer plano o background".to_string());
    }
    if memory_pct > 85.0 {
        causes.push("Consumo elevado de RAM con riesgo de paginacion".to_string());
    }
    if causes.is_empty() {
        causes.push(
            "No se detectan cuellos criticos inmediatos; posible degradacion progresiva"
                .to_string(),
        );
    }
    causes
}

pub fn run_kernel_slowpc_diagnostic() -> ToolResult {
    if let Err(e) = ensure_phase6_dir() {
        return ToolResult {
            tool_name: "run_kernel_slowpc_diagnostic".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    let sys_raw = sysinfo_tool::get_system_info_json();
    let sys = serde_json::from_str::<serde_json::Value>(&sys_raw).unwrap_or_else(|_| json!({}));

    let cpu = sys["cpu_usage"].as_f64().unwrap_or(0.0);
    let memory_used = sys["memory_used"].as_f64().unwrap_or(0.0);
    let memory_total = sys["memory_total"].as_f64().unwrap_or(1.0);
    let memory_pct = if memory_total > 0.0 {
        (memory_used / memory_total) * 100.0
    } else {
        0.0
    };

    let top_processes_raw = processes::list_processes("memory", 5);
    let top_processes = serde_json::from_str::<serde_json::Value>(&top_processes_raw.output)
        .unwrap_or_else(|_| json!([]));

    let causes = cpu_memory_causes(cpu, memory_pct);
    let actions = vec![
        "Ejecutar limpieza temporal y cache para reducir presion de disco".to_string(),
        "Revisar top procesos y cerrar/ajustar los no criticos".to_string(),
        "Programar mantenimiento proactivo para deteccion temprana".to_string(),
    ];

    let result = json!({
        "category": "slow_pc",
        "health_hint": {
            "cpu_pct": cpu,
            "memory_pct": memory_pct
        },
        "probable_causes": causes,
        "recommended_actions": actions,
        "top_processes": top_processes
    });

    let record = KernelDiagnosisRecord {
        id: format!(
            "KD-{}",
            uuid::Uuid::new_v4().simple().to_string().to_uppercase()
        ),
        created_at: Utc::now(),
        category: "slow_pc".to_string(),
        summary: "Diagnostico de PC lenta ejecutado".to_string(),
        details: result.clone(),
    };
    let _ = append_jsonl(&diagnostics_path(), &record);

    ToolResult {
        tool_name: "run_kernel_slowpc_diagnostic".into(),
        success: true,
        output: result.to_string(),
        error: None,
    }
}

pub fn run_kernel_network_playbook() -> ToolResult {
    if let Err(e) = ensure_phase6_dir() {
        return ToolResult {
            tool_name: "run_kernel_network_playbook".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    let net = network_diagnostic::run_network_diagnostic();
    let net_json = serde_json::from_str::<serde_json::Value>(&net.output)
        .unwrap_or_else(|_| json!({ "raw": net.output }));

    let playbook = json!({
        "category": "network",
        "steps": [
            "Verificar conectividad local e interfaz activa",
            "Validar resolucion DNS y latencia a endpoints criticos",
            "Identificar perdida de paquetes o degradacion intermitente"
        ],
        "diagnostic": net_json,
        "recommended_actions": [
            "Reiniciar gateway o interfaz en caso de inestabilidad",
            "Cambiar DNS temporalmente para validar resolucion",
            "Escalar a soporte si hay perdida sostenida"
        ]
    });

    let record = KernelDiagnosisRecord {
        id: format!(
            "KD-{}",
            uuid::Uuid::new_v4().simple().to_string().to_uppercase()
        ),
        created_at: Utc::now(),
        category: "network".to_string(),
        summary: "Playbook de diagnostico de red ejecutado".to_string(),
        details: playbook.clone(),
    };
    let _ = append_jsonl(&diagnostics_path(), &record);

    ToolResult {
        tool_name: "run_kernel_network_playbook".into(),
        success: true,
        output: playbook.to_string(),
        error: None,
    }
}

pub fn validate_kernel_guardrails(args: &serde_json::Value) -> ToolResult {
    let command = args
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap_or("Borra todos los archivos del sistema");

    let checks = vec![
        command.to_string(),
        "rm -rf /".to_string(),
        "del /f /s /q C:\\Windows\\*".to_string(),
        "format c:".to_string(),
        "Get-Process".to_string(),
    ];

    let mut blocked = 0usize;
    let mut details = Vec::new();

    for c in checks {
        let res = security::validate_command(&c);
        let is_blocked = res.is_err();
        if is_blocked {
            blocked += 1;
        }
        details.push(json!({
            "command": c,
            "blocked": is_blocked,
            "reason": res.err()
        }));
    }

    ToolResult {
        tool_name: "validate_kernel_guardrails".into(),
        success: true,
        output: json!({
            "blocked_count": blocked,
            "total_checked": details.len(),
            "checks": details
        })
        .to_string(),
        error: None,
    }
}

pub async fn run_kernel_autonomous_workflow(
    app: &tauri::AppHandle,
    role: crate::tools::rbac::UserRole,
    execute_actions: bool,
) -> ToolResult {
    if let Err(e) = ensure_phase6_dir() {
        return ToolResult {
            tool_name: "run_kernel_autonomous_workflow".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    let health_raw = phase2::health_overview();
    let health =
        serde_json::from_str::<serde_json::Value>(&health_raw.output).unwrap_or_else(|_| json!({}));
    let risk = health["risk"].as_str().unwrap_or("bajo");
    let cpu = health["current"]["cpu_pct"].as_f64().unwrap_or(0.0);
    let memory = health["current"]["memory_pct"].as_f64().unwrap_or(0.0);
    let disk = health["current"]["disk_pct"].as_f64().unwrap_or(0.0);

    let mut rules_triggered = Vec::new();
    let mut planned_actions = Vec::new();

    if disk > 90.0 {
        rules_triggered.push("disk_gt_90".to_string());
        planned_actions.push("run_cleanup".to_string());
    }
    if memory > 90.0 || cpu > 92.0 {
        rules_triggered.push("resource_pressure".to_string());
        planned_actions.push("run_kernel_slowpc_diagnostic".to_string());
    }
    if risk == "alto" {
        rules_triggered.push("risk_high".to_string());
        planned_actions.push("run_kernel_network_playbook".to_string());
    }

    let mut executed = Vec::new();
    if execute_actions {
        for action in &planned_actions {
            let result = match action.as_str() {
                "run_cleanup" => crate::tools::cleanup::run_cleanup(None).await,
                "run_kernel_slowpc_diagnostic" => run_kernel_slowpc_diagnostic(),
                "run_kernel_network_playbook" => run_kernel_network_playbook(),
                _ => ToolResult {
                    tool_name: action.clone(),
                    success: false,
                    output: String::new(),
                    error: Some("Accion no soportada".to_string()),
                },
            };
            executed.push(json!({
                "action": action,
                "success": result.success,
                "error": result.error
            }));
        }
    }

    let post_health_raw = phase2::health_overview();
    let post_health = serde_json::from_str::<serde_json::Value>(&post_health_raw.output)
        .unwrap_or_else(|_| json!({}));
    let post_risk = post_health["risk"].as_str().unwrap_or("bajo");

    let escalated_ticket_id = if post_risk == "alto" {
        let created = phase2::create_incident_ticket(&json!({
            "title": "Escalamiento automatico Fase 6",
            "category": "autohealing",
            "severity": "high",
            "details": format!("Riesgo sigue alto tras workflow autonomo. execute_actions={}", execute_actions),
            "source": "phase6-autonomous"
        }));
        serde_json::from_str::<serde_json::Value>(&created.output)
            .ok()
            .and_then(|v| v["id"].as_str().map(|s| s.to_string()))
    } else {
        None
    };

    let run = KernelAutomationRun {
        id: format!(
            "KA-{}",
            uuid::Uuid::new_v4().simple().to_string().to_uppercase()
        ),
        created_at: Utc::now(),
        mode: if execute_actions {
            "execute"
        } else {
            "simulate"
        }
        .to_string(),
        rules_triggered: rules_triggered.clone(),
        actions_planned: planned_actions.clone(),
        actions_executed: executed.clone(),
        post_verification: json!({
            "pre_risk": risk,
            "post_risk": post_risk,
            "pre_health": health,
            "post_health": post_health
        }),
        escalated_ticket_id: escalated_ticket_id.clone(),
    };
    let _ = append_jsonl(&automation_runs_path(), &run);

    let _ = phase2::run_automation_cycle(app, role, execute_actions).await;

    ToolResult {
        tool_name: "run_kernel_autonomous_workflow".into(),
        success: true,
        output: json!({
            "mode": run.mode,
            "rules_triggered": rules_triggered,
            "actions_planned": planned_actions,
            "actions_executed": executed,
            "post_verification": run.post_verification,
            "escalated_ticket_id": escalated_ticket_id
        })
        .to_string(),
        error: None,
    }
}

pub fn list_kernel_automation_runs(args: &serde_json::Value) -> ToolResult {
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
    let rows = read_automation_runs(limit);

    ToolResult {
        tool_name: "list_kernel_automation_runs".into(),
        success: true,
        output: serde_json::to_string(&rows).unwrap_or_else(|_| "[]".to_string()),
        error: None,
    }
}

pub fn list_kernel_diagnostics(args: &serde_json::Value) -> ToolResult {
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
    let rows = read_diagnostics(limit);

    ToolResult {
        tool_name: "list_kernel_diagnostics".into(),
        success: true,
        output: serde_json::to_string(&rows).unwrap_or_else(|_| "[]".to_string()),
        error: None,
    }
}

pub fn generate_kernelia_readiness_report() -> ToolResult {
    if let Err(e) = ensure_phase6_dir() {
        return ToolResult {
            tool_name: "generate_kernelia_readiness_report".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    let recent = read_diagnostics(20);
    let automations = read_automation_runs(20);
    let guardrails = validate_kernel_guardrails(&json!({}));
    let health = phase2::health_overview();

    let markdown = format!(
        "# KernelIA Readiness Report\n\nGenerado: {}\n\n## Diagnosticos recientes\n\n```json\n{}\n```\n\n## Automatizaciones recientes\n\n```json\n{}\n```\n\n## Guardrails de seguridad\n\n```json\n{}\n```\n\n## Salud operacional actual\n\n```json\n{}\n```\n",
        Utc::now().to_rfc3339(),
        serde_json::to_string_pretty(&recent).unwrap_or_else(|_| "[]".to_string()),
        serde_json::to_string_pretty(&automations).unwrap_or_else(|_| "[]".to_string()),
        serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(&guardrails.output).unwrap_or_else(|_| json!({})))
            .unwrap_or_else(|_| "{}".to_string()),
        serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(&health.output).unwrap_or_else(|_| json!({})))
            .unwrap_or_else(|_| "{}".to_string())
    );

    match fs::write(readiness_path(), markdown) {
        Ok(_) => ToolResult {
            tool_name: "generate_kernelia_readiness_report".into(),
            success: true,
            output: format!("Reporte generado en: {}", readiness_path().display()),
            error: None,
        },
        Err(e) => ToolResult {
            tool_name: "generate_kernelia_readiness_report".into(),
            success: false,
            output: String::new(),
            error: Some(e.to_string()),
        },
    }
}

pub fn run_phase6_smoke() -> ToolResult {
    let mut steps = Vec::new();

    let slowpc = run_kernel_slowpc_diagnostic();
    steps.push(json!({
        "step": "run_kernel_slowpc_diagnostic",
        "success": slowpc.success,
        "error": slowpc.error,
        "output": slowpc.output
    }));

    let net = run_kernel_network_playbook();
    steps.push(json!({
        "step": "run_kernel_network_playbook",
        "success": net.success,
        "error": net.error,
        "output": net.output
    }));

    let guardrails = validate_kernel_guardrails(&json!({}));
    steps.push(json!({
        "step": "validate_kernel_guardrails",
        "success": guardrails.success,
        "error": guardrails.error,
        "output": guardrails.output
    }));

    let automations = list_kernel_automation_runs(&json!({ "limit": 5 }));
    steps.push(json!({
        "step": "list_kernel_automation_runs",
        "success": automations.success,
        "error": automations.error,
        "output": automations.output
    }));

    let readiness = generate_kernelia_readiness_report();
    steps.push(json!({
        "step": "generate_kernelia_readiness_report",
        "success": readiness.success,
        "error": readiness.error,
        "output": readiness.output
    }));

    let ok = steps
        .iter()
        .all(|s| s.get("success").and_then(|v| v.as_bool()).unwrap_or(false));

    ToolResult {
        tool_name: "run_phase6_smoke".into(),
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
            Some("Uno o mas pasos del smoke E2E de Fase 6 fallaron.".to_string())
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guardrails_blocks_destructive_patterns() {
        let result = validate_kernel_guardrails(&json!({
            "command": "format c:"
        }));
        assert!(result.success);
        let payload = serde_json::from_str::<serde_json::Value>(&result.output)
            .expect("validate_kernel_guardrails should return JSON payload");
        let blocked = payload["blocked_count"].as_u64().unwrap_or(0);
        assert!(blocked >= 1);
    }

    #[test]
    fn slowpc_diagnostic_returns_causes() {
        let result = run_kernel_slowpc_diagnostic();
        assert!(result.success);
        let payload = serde_json::from_str::<serde_json::Value>(&result.output)
            .expect("run_kernel_slowpc_diagnostic should return JSON payload");
        assert!(payload["probable_causes"].is_array());
    }

    #[test]
    fn automation_runs_list_is_json() {
        let result = list_kernel_automation_runs(&json!({ "limit": 5 }));
        assert!(result.success);
        let payload = serde_json::from_str::<serde_json::Value>(&result.output)
            .expect("list_kernel_automation_runs should return JSON payload");
        assert!(payload.is_array());
    }
}
