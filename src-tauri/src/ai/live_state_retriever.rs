use crate::core;
use crate::rag::models::DomainSpecialty;
use crate::tools::{drivers, network_diagnostic, phase2, sysinfo_tool, windows_services};
use serde_json::{json, Value};

#[derive(Debug, Clone, Default)]
pub struct LiveStateContext {
    pub specialty: Option<DomainSpecialty>,
    pub summary: Vec<String>,
    pub observations: Vec<String>,
    pub conflict_flags: Vec<String>,
    pub snapshot_source: Option<String>,
    pub current_state: Value,
    pub last_snapshot: Option<Value>,
}

pub fn retrieve_live_state(specialty: DomainSpecialty) -> LiveStateContext {
    let last_snapshot = latest_core_snapshot();
    let snapshot_source = last_snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.get("source").and_then(Value::as_str))
        .map(str::to_string);

    let current_state = probe_current_state(&specialty);
    let observations = build_observations(&specialty, &current_state);
    let conflict_flags = detect_conflicts(&specialty, &current_state, last_snapshot.as_ref());

    let mut summary = Vec::new();
    if let Some(source) = &snapshot_source {
        summary.push(format!("snapshot_reciente={}", source));
    }
    summary.extend(observations.iter().cloned());
    summary.extend(conflict_flags.iter().map(|flag| format!("conflict={}", flag)));

    LiveStateContext {
        specialty: Some(specialty),
        summary,
        observations,
        conflict_flags,
        snapshot_source,
        current_state,
        last_snapshot,
    }
}

fn latest_core_snapshot() -> Option<Value> {
    core::list_snapshots(1)
        .into_iter()
        .next()
        .map(|snapshot| {
            json!({
                "id": snapshot.id,
                "timestamp": snapshot.timestamp,
                "source": snapshot.source,
                "data": snapshot.data
            })
        })
}

fn probe_current_state(specialty: &DomainSpecialty) -> Value {
    match specialty {
        DomainSpecialty::Network => {
            let diag = serde_json::from_str::<Value>(&network_diagnostic::run_network_diagnostic_json())
                .unwrap_or_else(|_| json!({}));
            json!({
                "specialty": "network",
                "network_diagnostic": diag
            })
        }
        DomainSpecialty::Services => {
            let spooler = windows_services::get_service_info("Spooler");
            json!({
                "specialty": "services",
                "spooler": {
                    "success": spooler.success,
                    "output": spooler.output,
                    "error": spooler.error
                }
            })
        }
        DomainSpecialty::Drivers => {
            let driver_issues = drivers::list_driver_issues();
            json!({
                "specialty": "drivers",
                "driver_issues": {
                    "success": driver_issues.success,
                    "output": driver_issues.output,
                    "error": driver_issues.error
                }
            })
        }
        DomainSpecialty::Security => {
            let health = phase2::health_summary();
            json!({
                "specialty": "security",
                "health_summary": {
                    "success": health.success,
                    "output": health.output,
                    "error": health.error
                }
            })
        }
        DomainSpecialty::Performance => build_performance_state(),
        DomainSpecialty::Maintenance => build_performance_state(),
        DomainSpecialty::System | DomainSpecialty::Telemetry | DomainSpecialty::Filesystem | DomainSpecialty::Software => {
            build_performance_state()
        }
        _ => {
            let sys = serde_json::from_str::<Value>(&sysinfo_tool::get_system_info_json())
                .unwrap_or_else(|_| json!({}));
            json!({
                "specialty": "generic",
                "system": sys
            })
        }
    }
}

fn build_performance_state() -> Value {
    let sys = serde_json::from_str::<Value>(&sysinfo_tool::get_system_info_json())
        .unwrap_or_else(|_| json!({}));
    let health = phase2::health_summary();
    json!({
        "specialty": "performance",
        "system": sys,
        "health_summary": {
            "success": health.success,
            "output": health.output,
            "error": health.error
        }
    })
}

fn build_observations(specialty: &DomainSpecialty, current_state: &Value) -> Vec<String> {
    match specialty {
        DomainSpecialty::Performance | DomainSpecialty::Maintenance => {
            let system = current_state.get("system").cloned().unwrap_or(Value::Null);
            let cpu = system.get("cpu_usage").and_then(Value::as_f64).unwrap_or(0.0);
            let memory_total = system.get("memory_total").and_then(Value::as_f64).unwrap_or(1.0);
            let memory_used = system.get("memory_used").and_then(Value::as_f64).unwrap_or(0.0);
            let memory_pct = if memory_total > 0.0 {
                (memory_used / memory_total) * 100.0
            } else {
                0.0
            };
            let disk_pct = system
                .get("disks")
                .and_then(Value::as_array)
                .and_then(|disks| disks.first())
                .and_then(|disk| {
                    let used = disk.get("used_space").and_then(Value::as_f64).unwrap_or(0.0);
                    let total = disk.get("total_space").and_then(Value::as_f64).unwrap_or(1.0);
                    if total > 0.0 {
                        Some((used / total) * 100.0)
                    } else {
                        None
                    }
                })
                .unwrap_or(0.0);

            let mut out = vec![
                format!("cpu_pct={:.1}", cpu),
                format!("memory_pct={:.1}", memory_pct),
                format!("disk_pct={:.1}", disk_pct),
            ];
            if cpu >= 90.0 {
                out.push("cpu_hot".to_string());
            }
            if memory_pct >= 90.0 {
                out.push("memory_pressure".to_string());
            }
            if disk_pct >= 95.0 {
                out.push("disk_critical".to_string());
            }
            out
        }
        DomainSpecialty::Network => {
            let diag = current_state
                .get("network_diagnostic")
                .cloned()
                .unwrap_or(Value::Null);
            let mut out = Vec::new();
            let snapshot = diag.get("snapshot").cloned().unwrap_or(Value::Null);
            if let Some(local_ip) = snapshot.get("local_ip_to_internet").and_then(Value::as_str) {
                out.push(format!("local_ip={}", local_ip));
            }
            if let Some(checks) = diag.get("checks").and_then(Value::as_array) {
                let failed = checks
                    .iter()
                    .filter(|item| item.get("success").and_then(Value::as_bool) == Some(false))
                    .count();
                out.push(format!("network_failed_checks={}", failed));
                if failed > 0 {
                    out.push("network_degraded".to_string());
                }
            }
            out
        }
        DomainSpecialty::Services => {
            let output = current_state
                .get("spooler")
                .and_then(|value| value.get("output"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_lowercase();
            let mut out = Vec::new();
            if output.contains("running") {
                out.push("spooler_running".to_string());
            }
            if output.contains("stopped") || output.contains("detenido") {
                out.push("spooler_stopped".to_string());
            }
            if out.is_empty() {
                out.push("spooler_unknown".to_string());
            }
            out
        }
        DomainSpecialty::Drivers => {
            let output = current_state
                .get("driver_issues")
                .and_then(|value| value.get("output"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_lowercase();
            let mut out = Vec::new();
            if output.contains("codigo: 43") || output.contains("code 43") {
                out.push("driver_code_43_live".to_string());
            }
            if output.contains("no se detectaron controladores con problemas") {
                out.push("drivers_ok".to_string());
            }
            if out.is_empty() {
                out.push("driver_state_unclear".to_string());
            }
            out
        }
        DomainSpecialty::Security => {
            let output = current_state
                .get("health_summary")
                .and_then(|value| value.get("output"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_lowercase();
            let mut out = Vec::new();
            if output.contains("riesgo operacional: alto") {
                out.push("security_risk_high".to_string());
            }
            if output.contains("anomalia detectada: si") {
                out.push("security_anomaly_present".to_string());
            }
            if out.is_empty() {
                out.push("security_state_nominal".to_string());
            }
            out
        }
        _ => vec!["generic_state_loaded".to_string()],
    }
}

fn detect_conflicts(
    specialty: &DomainSpecialty,
    current_state: &Value,
    last_snapshot: Option<&Value>,
) -> Vec<String> {
    let mut conflicts = Vec::new();

    if let Some(snapshot) = last_snapshot {
        let snapshot_data = snapshot.get("data").cloned().unwrap_or(Value::Null);
        match specialty {
            DomainSpecialty::Performance | DomainSpecialty::Maintenance => {
                let live_cpu = current_state
                    .get("system")
                    .and_then(|value| value.get("cpu_usage"))
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0);
                let snap_cpu = snapshot_data.get("cpu_usage").and_then(Value::as_f64).unwrap_or(live_cpu);
                if (live_cpu - snap_cpu).abs() >= 25.0 {
                    conflicts.push("cpu_shift_vs_snapshot".to_string());
                }
            }
            DomainSpecialty::Network => {
                let live_ip = current_state
                    .get("network_diagnostic")
                    .and_then(|diag| diag.get("snapshot"))
                    .and_then(|snap| snap.get("local_ip_to_internet"))
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let snap_ip = snapshot_data
                    .get("network")
                    .and_then(|network| network.get("local_ip"))
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                if !live_ip.is_empty() && !snap_ip.is_empty() && live_ip != snap_ip {
                    conflicts.push("network_identity_changed".to_string());
                }
            }
            DomainSpecialty::Services => {
                let live_output = current_state
                    .get("spooler")
                    .and_then(|value| value.get("output"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_lowercase();
                let snap_status = snapshot_data
                    .get("services")
                    .and_then(|value| value.get("spooler"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_lowercase();
                if !snap_status.is_empty()
                    && ((live_output.contains("running") && snap_status.contains("stopped"))
                        || (live_output.contains("stopped") && snap_status.contains("running")))
                {
                    conflicts.push("service_state_changed".to_string());
                }
            }
            DomainSpecialty::Drivers => {
                let live_output = current_state
                    .get("driver_issues")
                    .and_then(|value| value.get("output"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_lowercase();
                let snap_issue = snapshot_data
                    .get("drivers")
                    .and_then(|value| value.get("issue_code"))
                    .and_then(Value::as_i64)
                    .unwrap_or_default();
                if live_output.contains("codigo: 43") && snap_issue != 43 {
                    conflicts.push("driver_issue_changed".to_string());
                }
            }
            _ => {}
        }
    }

    conflicts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_performance_conflict_against_snapshot() {
        let current = json!({
            "system": { "cpu_usage": 95.0 }
        });
        let snapshot = json!({
            "data": { "cpu_usage": 30.0 }
        });

        let conflicts = detect_conflicts(
            &DomainSpecialty::Performance,
            &current,
            Some(&snapshot),
        );

        assert!(conflicts.iter().any(|item| item == "cpu_shift_vs_snapshot"));
    }

    #[test]
    fn extracts_driver_observation_for_code_43() {
        let current = json!({
            "driver_issues": {
                "output": "1. GPU NVIDIA\n   - Codigo: 43 (Dispositivo detenido por error)\n"
            }
        });

        let observations = build_observations(&DomainSpecialty::Drivers, &current);
        assert!(observations.iter().any(|item| item == "driver_code_43_live"));
    }

    #[test]
    fn extracts_service_observation_for_spooler_running() {
        let current = json!({
            "spooler": {
                "output": "STATE              : 4  RUNNING"
            }
        });

        let observations = build_observations(&DomainSpecialty::Services, &current);
        assert!(observations.iter().any(|item| item == "spooler_running"));
    }

    #[test]
    fn detects_service_conflict_against_snapshot() {
        let current = json!({
            "spooler": {
                "output": "STATE              : 4  RUNNING"
            }
        });
        let snapshot = json!({
            "data": {
                "services": {
                    "spooler": "stopped"
                }
            }
        });

        let conflicts = detect_conflicts(&DomainSpecialty::Services, &current, Some(&snapshot));
        assert!(conflicts.iter().any(|item| item == "service_state_changed"));
    }
}
