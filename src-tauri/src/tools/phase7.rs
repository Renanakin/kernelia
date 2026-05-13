use crate::tools::{network_diagnostic, processes, sysinfo_tool, ToolResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PerformanceSample {
    id: String,
    timestamp: DateTime<Utc>,
    category: String,
    target: String,
    iterations: usize,
    avg_ms: f64,
    p95_ms: f64,
    min_ms: f64,
    max_ms: f64,
    success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SaasLicense {
    id: String,
    tenant_id: String,
    plan: String,
    seats: u32,
    status: String,
    created_at: DateTime<Utc>,
}

fn phase7_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("nexus-lite")
        .join("phase7")
}

fn ensure_phase7_dir() -> Result<(), String> {
    fs::create_dir_all(phase7_dir()).map_err(|e| e.to_string())
}

fn samples_path() -> PathBuf {
    phase7_dir().join("performance_samples.jsonl")
}

fn report_path() -> PathBuf {
    phase7_dir().join("performance_report.md")
}

fn noc_report_path() -> PathBuf {
    phase7_dir().join("enterprise_noc_report.md")
}

fn saas_licenses_path() -> PathBuf {
    phase7_dir().join("saas_licenses.json")
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

fn read_samples(limit: usize) -> Vec<PerformanceSample> {
    let path = samples_path();
    if !path.exists() {
        return vec![];
    }

    let content = fs::read_to_string(path).unwrap_or_default();
    let mut rows: Vec<PerformanceSample> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<PerformanceSample>(line).ok())
        .collect();
    rows.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    rows.into_iter().take(limit).collect()
}

fn read_saas_licenses() -> Vec<SaasLicense> {
    let path = saas_licenses_path();
    if !path.exists() {
        return vec![];
    }
    let content = fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str::<Vec<SaasLicense>>(&content).unwrap_or_default()
}

fn save_saas_licenses(rows: &[SaasLicense]) -> Result<(), String> {
    ensure_phase7_dir()?;
    let payload = serde_json::to_string_pretty(rows).map_err(|e| e.to_string())?;
    fs::write(saas_licenses_path(), payload).map_err(|e| e.to_string())
}

fn summarize_durations_ms(
    durations: &[f64],
    successes: usize,
    total: usize,
) -> (f64, f64, f64, f64, f64) {
    if durations.is_empty() || total == 0 {
        return (0.0, 0.0, 0.0, 0.0, 0.0);
    }

    let mut sorted = durations.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let sum: f64 = sorted.iter().sum();
    let avg = sum / sorted.len() as f64;
    let min = *sorted.first().unwrap_or(&0.0);
    let max = *sorted.last().unwrap_or(&0.0);
    let idx95 = ((sorted.len() as f64) * 0.95).ceil() as usize;
    let p95 = sorted
        .get(idx95.saturating_sub(1).min(sorted.len().saturating_sub(1)))
        .copied()
        .unwrap_or(0.0);
    let success_rate = (successes as f64 / total as f64) * 100.0;

    (avg, p95, min, max, success_rate)
}

fn save_sample(
    category: &str,
    target: &str,
    iterations: usize,
    durations: &[f64],
    successes: usize,
) {
    let (avg, p95, min, max, success_rate) =
        summarize_durations_ms(durations, successes, iterations);

    let sample = PerformanceSample {
        id: format!(
            "PERF-{}",
            uuid::Uuid::new_v4().simple().to_string().to_uppercase()
        ),
        timestamp: Utc::now(),
        category: category.to_string(),
        target: target.to_string(),
        iterations,
        avg_ms: avg,
        p95_ms: p95,
        min_ms: min,
        max_ms: max,
        success_rate,
    };

    let _ = append_jsonl(&samples_path(), &sample);
}

pub fn run_latency_probe(args: &serde_json::Value) -> ToolResult {
    if let Err(e) = ensure_phase7_dir() {
        return ToolResult {
            tool_name: "run_latency_probe".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    let iterations = args
        .get("iterations")
        .and_then(|v| v.as_u64())
        .unwrap_or(10)
        .clamp(1, 200) as usize;

    let mut durations = Vec::with_capacity(iterations);
    let mut successes = 0usize;

    for _ in 0..iterations {
        let start = Instant::now();
        let raw = sysinfo_tool::get_system_info_json();
        let ok = serde_json::from_str::<serde_json::Value>(&raw).is_ok();
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        durations.push(elapsed_ms);
        if ok {
            successes += 1;
        }
    }

    let (avg, p95, min, max, success_rate) =
        summarize_durations_ms(&durations, successes, iterations);
    save_sample(
        "latency_probe",
        "get_system_info_json",
        iterations,
        &durations,
        successes,
    );

    ToolResult {
        tool_name: "run_latency_probe".into(),
        success: true,
        output: json!({
            "iterations": iterations,
            "avg_ms": avg,
            "p95_ms": p95,
            "min_ms": min,
            "max_ms": max,
            "success_rate": success_rate
        })
        .to_string(),
        error: None,
    }
}

pub fn run_tool_benchmark(args: &serde_json::Value) -> ToolResult {
    if let Err(e) = ensure_phase7_dir() {
        return ToolResult {
            tool_name: "run_tool_benchmark".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    let tool = args
        .get("tool")
        .and_then(|v| v.as_str())
        .unwrap_or("list_processes");
    let iterations = args
        .get("iterations")
        .and_then(|v| v.as_u64())
        .unwrap_or(5)
        .clamp(1, 100) as usize;

    let mut durations = Vec::with_capacity(iterations);
    let mut successes = 0usize;

    for _ in 0..iterations {
        let start = Instant::now();
        let ok = match tool {
            "get_system_info" => {
                serde_json::from_str::<serde_json::Value>(&sysinfo_tool::get_system_info_json())
                    .is_ok()
            }
            "list_processes" => {
                let res = processes::list_processes("memory", 20);
                res.success
            }
            "run_network_diagnostic" => {
                let res = network_diagnostic::run_network_diagnostic();
                res.success
            }
            _ => {
                return ToolResult {
                    tool_name: "run_tool_benchmark".into(),
                    success: false,
                    output: String::new(),
                    error: Some(format!("Tool '{}' no soportada para benchmark.", tool)),
                }
            }
        };
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        durations.push(elapsed_ms);
        if ok {
            successes += 1;
        }
    }

    let (avg, p95, min, max, success_rate) =
        summarize_durations_ms(&durations, successes, iterations);
    save_sample("tool_benchmark", tool, iterations, &durations, successes);

    ToolResult {
        tool_name: "run_tool_benchmark".into(),
        success: true,
        output: json!({
            "tool": tool,
            "iterations": iterations,
            "avg_ms": avg,
            "p95_ms": p95,
            "min_ms": min,
            "max_ms": max,
            "success_rate": success_rate
        })
        .to_string(),
        error: None,
    }
}

pub fn get_performance_kpis(args: &serde_json::Value) -> ToolResult {
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(200) as usize;

    let rows = read_samples(limit);
    let total = rows.len();

    let avg_latency = if total == 0 {
        0.0
    } else {
        rows.iter().map(|r| r.avg_ms).sum::<f64>() / total as f64
    };

    let avg_p95 = if total == 0 {
        0.0
    } else {
        rows.iter().map(|r| r.p95_ms).sum::<f64>() / total as f64
    };

    let avg_success = if total == 0 {
        0.0
    } else {
        rows.iter().map(|r| r.success_rate).sum::<f64>() / total as f64
    };

    let worst = rows.iter().max_by(|a, b| {
        a.p95_ms
            .partial_cmp(&b.p95_ms)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    ToolResult {
        tool_name: "get_performance_kpis".into(),
        success: true,
        output: json!({
            "samples": total,
            "avg_latency_ms": (avg_latency * 100.0).round() / 100.0,
            "avg_p95_ms": (avg_p95 * 100.0).round() / 100.0,
            "avg_success_rate": (avg_success * 100.0).round() / 100.0,
            "worst_p95": worst.map(|w| json!({
                "category": w.category,
                "target": w.target,
                "p95_ms": w.p95_ms
            }))
        })
        .to_string(),
        error: None,
    }
}

pub fn generate_performance_report() -> ToolResult {
    if let Err(e) = ensure_phase7_dir() {
        return ToolResult {
            tool_name: "generate_performance_report".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    let kpis = get_performance_kpis(&json!({ "limit": 500 }));
    let recent = read_samples(50);

    let markdown = format!(
        "# Performance Report\n\nGenerado: {}\n\n## KPIs\n\n```json\n{}\n```\n\n## Muestras recientes\n\n```json\n{}\n```\n",
        Utc::now().to_rfc3339(),
        serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(&kpis.output).unwrap_or_else(|_| json!({})))
            .unwrap_or_else(|_| "{}".to_string()),
        serde_json::to_string_pretty(&recent).unwrap_or_else(|_| "[]".to_string())
    );

    match fs::write(report_path(), markdown) {
        Ok(_) => ToolResult {
            tool_name: "generate_performance_report".into(),
            success: true,
            output: format!("Reporte generado en: {}", report_path().display()),
            error: None,
        },
        Err(e) => ToolResult {
            tool_name: "generate_performance_report".into(),
            success: false,
            output: String::new(),
            error: Some(e.to_string()),
        },
    }
}

pub fn run_phase7_smoke() -> ToolResult {
    let mut steps = Vec::new();

    let probe = run_latency_probe(&json!({ "iterations": 10 }));
    steps.push(json!({
        "step": "run_latency_probe",
        "success": probe.success,
        "error": probe.error,
        "output": probe.output
    }));

    let bench = run_tool_benchmark(&json!({
        "tool": "list_processes",
        "iterations": 5
    }));
    steps.push(json!({
        "step": "run_tool_benchmark",
        "success": bench.success,
        "error": bench.error,
        "output": bench.output
    }));

    let kpis = get_performance_kpis(&json!({ "limit": 200 }));
    steps.push(json!({
        "step": "get_performance_kpis",
        "success": kpis.success,
        "error": kpis.error,
        "output": kpis.output
    }));

    let report = generate_performance_report();
    steps.push(json!({
        "step": "generate_performance_report",
        "success": report.success,
        "error": report.error,
        "output": report.output
    }));

    let noc = get_noc_global_status();
    steps.push(json!({
        "step": "get_noc_global_status",
        "success": noc.success,
        "error": noc.error,
        "output": noc.output
    }));

    let license = register_saas_license(&json!({
        "tenant_id": "TEN-SMOKE",
        "plan": "business",
        "seats": 10
    }));
    steps.push(json!({
        "step": "register_saas_license",
        "success": license.success,
        "error": license.error,
        "output": license.output
    }));

    let list_licenses = list_saas_licenses();
    steps.push(json!({
        "step": "list_saas_licenses",
        "success": list_licenses.success,
        "error": list_licenses.error,
        "output": list_licenses.output
    }));

    let ent_report = generate_enterprise_noc_report();
    steps.push(json!({
        "step": "generate_enterprise_noc_report",
        "success": ent_report.success,
        "error": ent_report.error,
        "output": ent_report.output
    }));

    let ok = steps
        .iter()
        .all(|s| s.get("success").and_then(|v| v.as_bool()).unwrap_or(false));

    ToolResult {
        tool_name: "run_phase7_smoke".into(),
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
            Some("Uno o más pasos del smoke E2E de Fase 7 fallaron.".to_string())
        },
    }
}

pub fn get_noc_global_status() -> ToolResult {
    let mt = crate::tools::phase3::cloud_multi_tenant_overview();
    let dash = crate::tools::phase5::get_enterprise_dashboard();
    let kpis = get_performance_kpis(&json!({ "limit": 200 }));

    let mt_json =
        serde_json::from_str::<serde_json::Value>(&mt.output).unwrap_or_else(|_| json!({}));
    let dash_json =
        serde_json::from_str::<serde_json::Value>(&dash.output).unwrap_or_else(|_| json!({}));
    let perf_json =
        serde_json::from_str::<serde_json::Value>(&kpis.output).unwrap_or_else(|_| json!({}));

    let tenant_count = mt_json["summary"]["tenant_count"].as_u64().unwrap_or(0);
    let high_risk_reports = dash_json["kpis"]["high_risk_reports"].as_u64().unwrap_or(0);
    let avg_success = perf_json["avg_success_rate"].as_f64().unwrap_or(0.0);

    let sla_status = if avg_success >= 99.0 {
        "healthy"
    } else if avg_success >= 95.0 {
        "warning"
    } else {
        "critical"
    };

    ToolResult {
        tool_name: "get_noc_global_status".into(),
        success: true,
        output: json!({
            "executed_at": Utc::now(),
            "tenant_count": tenant_count,
            "high_risk_reports": high_risk_reports,
            "avg_success_rate": avg_success,
            "sla_status": sla_status,
            "sources": {
                "multi_tenant": mt_json,
                "enterprise_dashboard": dash_json,
                "performance_kpis": perf_json
            }
        })
        .to_string(),
        error: None,
    }
}

pub fn register_saas_license(args: &serde_json::Value) -> ToolResult {
    if let Err(e) = ensure_phase7_dir() {
        return ToolResult {
            tool_name: "register_saas_license".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    let tenant_id = args
        .get("tenant_id")
        .and_then(|v| v.as_str())
        .unwrap_or("TEN-LOCAL")
        .trim()
        .to_string();
    let plan = args
        .get("plan")
        .and_then(|v| v.as_str())
        .unwrap_or("basic")
        .to_lowercase();
    let seats = args
        .get("seats")
        .and_then(|v| v.as_u64())
        .unwrap_or(1)
        .clamp(1, 10000) as u32;
    let status = args
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("active")
        .to_string();

    let mut rows = read_saas_licenses();
    if let Some(idx) = rows.iter().position(|r| r.tenant_id == tenant_id) {
        rows[idx].plan = plan;
        rows[idx].seats = seats;
        rows[idx].status = status;
        let updated = rows[idx].clone();
        if let Err(e) = save_saas_licenses(&rows) {
            return ToolResult {
                tool_name: "register_saas_license".into(),
                success: false,
                output: String::new(),
                error: Some(e),
            };
        }
        return ToolResult {
            tool_name: "register_saas_license".into(),
            success: true,
            output: json!({ "updated": true, "license": updated }).to_string(),
            error: None,
        };
    }

    let license = SaasLicense {
        id: format!(
            "LIC-{}",
            uuid::Uuid::new_v4().simple().to_string().to_uppercase()
        ),
        tenant_id,
        plan,
        seats,
        status,
        created_at: Utc::now(),
    };
    rows.push(license.clone());
    if let Err(e) = save_saas_licenses(&rows) {
        return ToolResult {
            tool_name: "register_saas_license".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }
    ToolResult {
        tool_name: "register_saas_license".into(),
        success: true,
        output: json!({ "updated": false, "license": license }).to_string(),
        error: None,
    }
}

pub fn list_saas_licenses() -> ToolResult {
    ToolResult {
        tool_name: "list_saas_licenses".into(),
        success: true,
        output: serde_json::to_string(&read_saas_licenses()).unwrap_or_else(|_| "[]".to_string()),
        error: None,
    }
}

pub fn generate_enterprise_noc_report() -> ToolResult {
    if let Err(e) = ensure_phase7_dir() {
        return ToolResult {
            tool_name: "generate_enterprise_noc_report".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }
    let noc = get_noc_global_status();
    let licenses = list_saas_licenses();
    let noc_json =
        serde_json::from_str::<serde_json::Value>(&noc.output).unwrap_or_else(|_| json!({}));
    let licenses_json =
        serde_json::from_str::<serde_json::Value>(&licenses.output).unwrap_or_else(|_| json!([]));

    let markdown = format!(
        "# Enterprise NOC Report\n\nGenerado: {}\n\n## NOC Global\n\n```json\n{}\n```\n\n## SaaS Licenses\n\n```json\n{}\n```\n",
        Utc::now().to_rfc3339(),
        serde_json::to_string_pretty(&noc_json).unwrap_or_else(|_| "{}".to_string()),
        serde_json::to_string_pretty(&licenses_json).unwrap_or_else(|_| "[]".to_string()),
    );

    match fs::write(noc_report_path(), markdown) {
        Ok(_) => ToolResult {
            tool_name: "generate_enterprise_noc_report".into(),
            success: true,
            output: format!("Reporte NOC generado en: {}", noc_report_path().display()),
            error: None,
        },
        Err(e) => ToolResult {
            tool_name: "generate_enterprise_noc_report".into(),
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
    fn latency_probe_returns_metrics() {
        let result = run_latency_probe(&json!({ "iterations": 5 }));
        assert!(result.success);
        let payload = serde_json::from_str::<serde_json::Value>(&result.output)
            .expect("run_latency_probe should return JSON payload");
        assert!(payload["avg_ms"].is_number());
        assert!(payload["p95_ms"].is_number());
    }

    #[test]
    fn performance_kpis_returns_structure() {
        let _ = run_latency_probe(&json!({ "iterations": 3 }));
        let result = get_performance_kpis(&json!({ "limit": 20 }));
        assert!(result.success);
        let payload = serde_json::from_str::<serde_json::Value>(&result.output)
            .expect("get_performance_kpis should return JSON payload");
        assert!(payload["samples"].is_number());
        assert!(payload["avg_latency_ms"].is_number());
    }

    #[test]
    fn noc_global_status_returns_sla() {
        let result = get_noc_global_status();
        assert!(result.success);
        let payload = serde_json::from_str::<serde_json::Value>(&result.output)
            .expect("get_noc_global_status should return JSON payload");
        assert!(payload["sla_status"].is_string());
    }
}
