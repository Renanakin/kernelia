use crate::tools::ToolResult;
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
struct PerformanceSampleRow {
    category: String,
    target: String,
    p95_ms: f64,
    avg_ms: f64,
    success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PredictedIncident {
    id: String,
    predicted_at: DateTime<Utc>,
    incident_type: String,
    probability: f64,
    severity: String,
    rationale: String,
    recommended_prevention: Vec<String>,
}

fn phase7_samples_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("nexus-lite")
        .join("phase7")
        .join("performance_samples.jsonl")
}

fn phase8_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("nexus-lite")
        .join("phase8")
}

fn ensure_phase8_dir() -> Result<(), String> {
    fs::create_dir_all(phase8_dir()).map_err(|e| e.to_string())
}

fn anomalies_path() -> PathBuf {
    phase8_dir().join("reliability_anomalies.jsonl")
}

fn report_path() -> PathBuf {
    phase8_dir().join("reliability_report.md")
}

fn predictions_path() -> PathBuf {
    phase8_dir().join("predicted_incidents.jsonl")
}

fn playbook_path() -> PathBuf {
    phase8_dir().join("autonomous_playbook.md")
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

fn read_phase7_samples(limit: usize) -> Vec<PerformanceSampleRow> {
    let path = phase7_samples_path();
    if !path.exists() {
        return vec![];
    }

    let content = fs::read_to_string(path).unwrap_or_default();
    content
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .map(|v| PerformanceSampleRow {
            category: v["category"].as_str().unwrap_or("unknown").to_string(),
            target: v["target"].as_str().unwrap_or("unknown").to_string(),
            p95_ms: v["p95_ms"].as_f64().unwrap_or(0.0),
            avg_ms: v["avg_ms"].as_f64().unwrap_or(0.0),
            success_rate: v["success_rate"].as_f64().unwrap_or(0.0),
        })
        .take(limit)
        .collect()
}

fn read_reliability_events(limit: usize) -> Vec<ReliabilityEvent> {
    let path = anomalies_path();
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

fn read_predictions(limit: usize) -> Vec<PredictedIncident> {
    let path = predictions_path();
    if !path.exists() {
        return vec![];
    }
    let content = fs::read_to_string(path).unwrap_or_default();
    let mut rows: Vec<PredictedIncident> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<PredictedIncident>(line).ok())
        .collect();
    rows.sort_by(|a, b| b.predicted_at.cmp(&a.predicted_at));
    rows.into_iter().take(limit).collect()
}

fn compute_baseline_p95(samples: &[PerformanceSampleRow]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.iter().map(|s| s.p95_ms).sum::<f64>() / samples.len() as f64
}

pub fn detect_performance_anomalies(args: &serde_json::Value) -> ToolResult {
    if let Err(e) = ensure_phase8_dir() {
        return ToolResult {
            tool_name: "detect_performance_anomalies".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(200) as usize;
    let p95_multiplier = args
        .get("p95_multiplier")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.8)
        .max(1.0);
    let min_success_rate = args
        .get("min_success_rate")
        .and_then(|v| v.as_f64())
        .unwrap_or(95.0)
        .clamp(1.0, 100.0);

    let samples = read_phase7_samples(limit);
    if samples.is_empty() {
        return ToolResult {
            tool_name: "detect_performance_anomalies".into(),
            success: true,
            output: json!({
                "total_samples": 0,
                "baseline_p95_ms": 0.0,
                "anomalies": []
            })
            .to_string(),
            error: None,
        };
    }

    let baseline = compute_baseline_p95(&samples);
    let mut anomalies = Vec::new();

    for sample in samples {
        let high_latency = baseline > 0.0 && sample.p95_ms > baseline * p95_multiplier;
        let low_success = sample.success_rate < min_success_rate;

        if high_latency || low_success {
            let severity = if sample.success_rate < 90.0
                || (baseline > 0.0 && sample.p95_ms > baseline * 2.5)
            {
                "high"
            } else {
                "medium"
            };

            let message = if high_latency && low_success {
                "Anomalía combinada: p95 elevada y éxito bajo".to_string()
            } else if high_latency {
                "Anomalía de latencia: p95 sobre umbral esperado".to_string()
            } else {
                "Anomalía de confiabilidad: tasa de éxito por debajo del objetivo".to_string()
            };

            let event = ReliabilityEvent {
                id: format!(
                    "REL-{}",
                    uuid::Uuid::new_v4().simple().to_string().to_uppercase()
                ),
                timestamp: Utc::now(),
                severity: severity.to_string(),
                category: sample.category,
                target: sample.target,
                observed_p95_ms: sample.p95_ms,
                baseline_p95_ms: baseline,
                success_rate: sample.success_rate,
                message,
            };

            let _ = append_jsonl(&anomalies_path(), &event);
            anomalies.push(event);
        }
    }

    ToolResult {
        tool_name: "detect_performance_anomalies".into(),
        success: true,
        output: json!({
            "total_samples": limit,
            "baseline_p95_ms": (baseline * 100.0).round() / 100.0,
            "anomaly_count": anomalies.len(),
            "anomalies": anomalies
        })
        .to_string(),
        error: None,
    }
}

pub fn calculate_sla_status(args: &serde_json::Value) -> ToolResult {
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(300) as usize;
    let target_success = args
        .get("target_success_rate")
        .and_then(|v| v.as_f64())
        .unwrap_or(99.0)
        .clamp(1.0, 100.0);

    let samples = read_phase7_samples(limit);
    if samples.is_empty() {
        return ToolResult {
            tool_name: "calculate_sla_status".into(),
            success: true,
            output: json!({
                "samples": 0,
                "target_success_rate": target_success,
                "actual_success_rate": 0.0,
                "sla_met": false,
                "status": "insufficient_data"
            })
            .to_string(),
            error: None,
        };
    }

    let actual = samples.iter().map(|s| s.success_rate).sum::<f64>() / samples.len() as f64;
    let sla_met = actual >= target_success;

    ToolResult {
        tool_name: "calculate_sla_status".into(),
        success: true,
        output: json!({
            "samples": samples.len(),
            "target_success_rate": target_success,
            "actual_success_rate": (actual * 100.0).round() / 100.0,
            "sla_met": sla_met,
            "status": if sla_met { "healthy" } else { "degraded" }
        })
        .to_string(),
        error: None,
    }
}

pub fn recommend_reliability_actions(args: &serde_json::Value) -> ToolResult {
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;

    let recent = read_reliability_events(limit);
    let high = recent.iter().filter(|e| e.severity == "high").count();
    let medium = recent.iter().filter(|e| e.severity == "medium").count();

    let mut actions: Vec<String> = Vec::new();
    if high > 0 {
        actions.push(
            "Elevar prioridad a P1 y abrir caso de soporte con evidencia de latencia/éxito."
                .to_string(),
        );
        actions.push("Reducir carga concurrente y ejecutar benchmark aislado para confirmar cuello de botella.".to_string());
    }
    if medium > 0 {
        actions.push(
            "Programar run_phase7_smoke cada 4 horas para captura temprana de regresiones."
                .to_string(),
        );
    }
    if actions.is_empty() {
        actions.push(
            "Mantener monitoreo activo; no se detectan anomalías relevantes en la ventana actual."
                .to_string(),
        );
    }

    ToolResult {
        tool_name: "recommend_reliability_actions".into(),
        success: true,
        output: json!({
            "window_events": recent.len(),
            "high_severity": high,
            "medium_severity": medium,
            "recommended_actions": actions
        })
        .to_string(),
        error: None,
    }
}

pub fn generate_reliability_report() -> ToolResult {
    if let Err(e) = ensure_phase8_dir() {
        return ToolResult {
            tool_name: "generate_reliability_report".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    let sla = calculate_sla_status(&json!({ "limit": 500, "target_success_rate": 99.0 }));
    let anomalies = detect_performance_anomalies(&json!({
        "limit": 250,
        "p95_multiplier": 1.8,
        "min_success_rate": 95.0
    }));
    let actions = recommend_reliability_actions(&json!({ "limit": 100 }));

    let markdown = format!(
        "# Reliability Report\n\nGenerado: {}\n\n## SLA Status\n\n```json\n{}\n```\n\n## Anomalías de Rendimiento\n\n```json\n{}\n```\n\n## Recomendaciones\n\n```json\n{}\n```\n",
        Utc::now().to_rfc3339(),
        serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(&sla.output).unwrap_or_else(|_| json!({})))
            .unwrap_or_else(|_| "{}".to_string()),
        serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(&anomalies.output).unwrap_or_else(|_| json!({})))
            .unwrap_or_else(|_| "{}".to_string()),
        serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(&actions.output).unwrap_or_else(|_| json!({})))
            .unwrap_or_else(|_| "{}".to_string())
    );

    match fs::write(report_path(), markdown) {
        Ok(_) => ToolResult {
            tool_name: "generate_reliability_report".into(),
            success: true,
            output: format!("Reporte generado en: {}", report_path().display()),
            error: None,
        },
        Err(e) => ToolResult {
            tool_name: "generate_reliability_report".into(),
            success: false,
            output: String::new(),
            error: Some(e.to_string()),
        },
    }
}

pub fn run_phase8_smoke() -> ToolResult {
    let mut steps = Vec::new();

    let anomalies = detect_performance_anomalies(&json!({
        "limit": 150,
        "p95_multiplier": 1.7,
        "min_success_rate": 95.0
    }));
    steps.push(json!({
        "step": "detect_performance_anomalies",
        "success": anomalies.success,
        "error": anomalies.error,
        "output": anomalies.output
    }));

    let sla = calculate_sla_status(&json!({
        "limit": 200,
        "target_success_rate": 99.0
    }));
    steps.push(json!({
        "step": "calculate_sla_status",
        "success": sla.success,
        "error": sla.error,
        "output": sla.output
    }));

    let actions = recommend_reliability_actions(&json!({ "limit": 100 }));
    steps.push(json!({
        "step": "recommend_reliability_actions",
        "success": actions.success,
        "error": actions.error,
        "output": actions.output
    }));

    let report = generate_reliability_report();
    steps.push(json!({
        "step": "generate_reliability_report",
        "success": report.success,
        "error": report.error,
        "output": report.output
    }));

    let prediction = predict_operational_incidents(&json!({ "limit": 120 }));
    steps.push(json!({
        "step": "predict_operational_incidents",
        "success": prediction.success,
        "error": prediction.error,
        "output": prediction.output
    }));

    let root_cause = explain_root_cause(&json!({ "limit": 120 }));
    steps.push(json!({
        "step": "explain_root_cause",
        "success": root_cause.success,
        "error": root_cause.error,
        "output": root_cause.output
    }));

    let playbook = generate_autonomous_playbook(&json!({ "limit": 120 }));
    steps.push(json!({
        "step": "generate_autonomous_playbook",
        "success": playbook.success,
        "error": playbook.error,
        "output": playbook.output
    }));

    let ok = steps
        .iter()
        .all(|s| s.get("success").and_then(|v| v.as_bool()).unwrap_or(false));

    ToolResult {
        tool_name: "run_phase8_smoke".into(),
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
            Some("Uno o más pasos del smoke E2E de Fase 8 fallaron.".to_string())
        },
    }
}

pub fn predict_operational_incidents(args: &serde_json::Value) -> ToolResult {
    if let Err(e) = ensure_phase8_dir() {
        return ToolResult {
            tool_name: "predict_operational_incidents".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(200) as usize;
    let samples = read_phase7_samples(limit);
    let anomalies = read_reliability_events(limit);

    let baseline = compute_baseline_p95(&samples);
    let anomaly_rate = if samples.is_empty() {
        0.0
    } else {
        anomalies.len() as f64 / samples.len() as f64
    };
    let avg_success = if samples.is_empty() {
        0.0
    } else {
        samples.iter().map(|s| s.success_rate).sum::<f64>() / samples.len() as f64
    };

    let mut predictions = Vec::new();

    let prob_latency =
        (anomaly_rate * 0.6 + if baseline > 0.0 { 0.25 } else { 0.05 }).clamp(0.05, 0.95);
    predictions.push(PredictedIncident {
        id: format!(
            "PRED-{}",
            uuid::Uuid::new_v4().simple().to_string().to_uppercase()
        ),
        predicted_at: Utc::now(),
        incident_type: "latency_degradation".to_string(),
        probability: prob_latency,
        severity: if prob_latency > 0.7 {
            "high".to_string()
        } else {
            "medium".to_string()
        },
        rationale: format!(
            "AnomalyRate={:.2} BaselineP95={:.2}",
            anomaly_rate, baseline
        ),
        recommended_prevention: vec![
            "Reducir carga concurrente en horarios pico".to_string(),
            "Ejecutar benchmark aislado de list_processes y network_diagnostic".to_string(),
        ],
    });

    let prob_sla = ((100.0 - avg_success).max(0.0) / 100.0 + anomaly_rate * 0.5).clamp(0.05, 0.95);
    predictions.push(PredictedIncident {
        id: format!(
            "PRED-{}",
            uuid::Uuid::new_v4().simple().to_string().to_uppercase()
        ),
        predicted_at: Utc::now(),
        incident_type: "sla_breach".to_string(),
        probability: prob_sla,
        severity: if prob_sla > 0.75 {
            "high".to_string()
        } else {
            "medium".to_string()
        },
        rationale: format!(
            "AvgSuccess={:.2} AnomalyRate={:.2}",
            avg_success, anomaly_rate
        ),
        recommended_prevention: vec![
            "Programar run_phase7_smoke cada 4 horas".to_string(),
            "Escalar incidentes repetitivos al equipo enterprise".to_string(),
        ],
    });

    for p in &predictions {
        let _ = append_jsonl(&predictions_path(), p);
    }

    ToolResult {
        tool_name: "predict_operational_incidents".into(),
        success: true,
        output: json!({
            "samples": samples.len(),
            "anomalies": anomalies.len(),
            "predictions": predictions
        })
        .to_string(),
        error: None,
    }
}

pub fn explain_root_cause(args: &serde_json::Value) -> ToolResult {
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
    let events = read_reliability_events(limit);

    if events.is_empty() {
        return ToolResult {
            tool_name: "explain_root_cause".into(),
            success: true,
            output: json!({
                "status": "insufficient_data",
                "root_cause_hypothesis": "No hay eventos suficientes para correlacion causal.",
                "confidence": 0.2
            })
            .to_string(),
            error: None,
        };
    }

    let high = events.iter().filter(|e| e.severity == "high").count();
    let avg_delta = events
        .iter()
        .map(|e| (e.observed_p95_ms - e.baseline_p95_ms).max(0.0))
        .sum::<f64>()
        / events.len() as f64;
    let avg_success = events.iter().map(|e| e.success_rate).sum::<f64>() / events.len() as f64;

    let (root_cause, confidence) = if high > 0 && avg_delta > 30.0 {
        (
            "Degradacion por saturacion de recursos y contencion de latencia en herramientas core",
            0.82,
        )
    } else if avg_success < 95.0 {
        (
            "Inestabilidad operativa con perdida intermitente de exito en ejecuciones",
            0.74,
        )
    } else {
        ("Variabilidad normal sin causa critica dominante", 0.58)
    };

    ToolResult {
        tool_name: "explain_root_cause".into(),
        success: true,
        output: json!({
            "events_analyzed": events.len(),
            "high_severity_events": high,
            "avg_latency_delta_ms": (avg_delta * 100.0).round() / 100.0,
            "avg_success_rate": (avg_success * 100.0).round() / 100.0,
            "root_cause_hypothesis": root_cause,
            "confidence": confidence
        })
        .to_string(),
        error: None,
    }
}

pub fn generate_autonomous_playbook(args: &serde_json::Value) -> ToolResult {
    if let Err(e) = ensure_phase8_dir() {
        return ToolResult {
            tool_name: "generate_autonomous_playbook".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(120) as usize;
    let predictions = read_predictions(limit);
    let root = explain_root_cause(&json!({ "limit": limit }));
    let root_json =
        serde_json::from_str::<serde_json::Value>(&root.output).unwrap_or_else(|_| json!({}));

    let mut steps = vec![
        "1. Ejecutar run_latency_probe y run_tool_benchmark para refrescar baseline.".to_string(),
        "2. Ejecutar detect_performance_anomalies con umbral p95 dinámico.".to_string(),
        "3. Si anomaly_count > 0, activar run_kernel_autonomous_workflow en modo simulación.".to_string(),
        "4. Si riesgo persiste, ejecutar run_kernel_autonomous_workflow con acciones y crear ticket.".to_string(),
        "5. Recalcular SLA y emitir reporte enterprise NOC.".to_string(),
    ];

    if predictions.iter().any(|p| p.probability > 0.75) {
        steps.push("6. Escalar a P1 y notificar a canal de incidentes enterprise.".to_string());
    }

    let markdown = format!(
        "# Autonomous Playbook Fase 8\n\nGenerado: {}\n\n## Predicciones\n\n```json\n{}\n```\n\n## Causa Raiz (explicable)\n\n```json\n{}\n```\n\n## Plan de ejecucion autonoma\n\n{}\n",
        Utc::now().to_rfc3339(),
        serde_json::to_string_pretty(&predictions).unwrap_or_else(|_| "[]".to_string()),
        serde_json::to_string_pretty(&root_json).unwrap_or_else(|_| "{}".to_string()),
        steps.join("\n")
    );

    match fs::write(playbook_path(), markdown) {
        Ok(_) => ToolResult {
            tool_name: "generate_autonomous_playbook".into(),
            success: true,
            output: format!("Playbook generado en: {}", playbook_path().display()),
            error: None,
        },
        Err(e) => ToolResult {
            tool_name: "generate_autonomous_playbook".into(),
            success: false,
            output: String::new(),
            error: Some(e.to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anomalies_detection_returns_json() {
        let _ = calculate_sla_status(&json!({ "limit": 10 }));
        let result = detect_performance_anomalies(&json!({
            "limit": 20,
            "p95_multiplier": 1.5,
            "min_success_rate": 99.5
        }));
        assert!(result.success);
        let payload = serde_json::from_str::<serde_json::Value>(&result.output)
            .expect("detect_performance_anomalies should return JSON payload");
        assert!(payload["baseline_p95_ms"].is_number());
    }

    #[test]
    fn sla_status_returns_structure() {
        let result = calculate_sla_status(&json!({
            "limit": 10,
            "target_success_rate": 95.0
        }));
        assert!(result.success);
        let payload = serde_json::from_str::<serde_json::Value>(&result.output)
            .expect("calculate_sla_status should return JSON payload");
        assert!(payload["target_success_rate"].is_number());
        assert!(payload["status"].is_string());
    }

    #[test]
    fn root_cause_explainer_returns_hypothesis() {
        let _ = detect_performance_anomalies(&json!({
            "limit": 10,
            "p95_multiplier": 1.2,
            "min_success_rate": 99.9
        }));
        let result = explain_root_cause(&json!({ "limit": 10 }));
        assert!(result.success);
        let payload = serde_json::from_str::<serde_json::Value>(&result.output)
            .expect("explain_root_cause should return JSON payload");
        assert!(payload["root_cause_hypothesis"].is_string());
    }
}
