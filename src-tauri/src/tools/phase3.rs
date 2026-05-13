use crate::config::AppSettings;
use crate::tools::{network_diagnostic, phase2, processes, ToolResult};
use chrono::{DateTime, Utc};
use ring::digest::{digest, SHA256};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EndpointRecord {
    id: String,
    hostname: String,
    os: String,
    site: String,
    last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TenantRecord {
    id: String,
    name: String,
    created_at: DateTime<Utc>,
    endpoints: Vec<EndpointRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RemoteSessionRecord {
    session_id: String,
    tenant_id: String,
    endpoint_id: String,
    operator: String,
    reason: String,
    transport: String,
    connect_uri: String,
    started_at: DateTime<Utc>,
    closed_at: Option<DateTime<Utc>>,
    status: String,
    outcome: Option<String>,
    evidence_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SnapshotRecord {
    id: String,
    created_at: DateTime<Utc>,
    reason: String,
    snapshot_dir: String,
    files_backed_up: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReleaseAttestation {
    id: String,
    file_path: String,
    sha256: String,
    authenticode_status: String,
    reputation: String,
    signed_at: DateTime<Utc>,
    signed_by: String,
}

fn phase3_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("nexus-lite")
        .join("phase3")
}

fn ensure_phase3_dir() -> Result<(), String> {
    fs::create_dir_all(phase3_dir()).map_err(|e| e.to_string())
}

fn tenants_path() -> PathBuf {
    phase3_dir().join("tenants.json")
}

fn sessions_path() -> PathBuf {
    phase3_dir().join("remote_sessions.jsonl")
}

fn snapshots_path() -> PathBuf {
    phase3_dir().join("rollback_snapshots.json")
}

fn attestations_path() -> PathBuf {
    phase3_dir().join("release_attestations.jsonl")
}

fn snapshots_dir() -> PathBuf {
    phase3_dir().join("snapshots")
}

fn tool_success(tool_name: &str, output: serde_json::Value) -> ToolResult {
    ToolResult {
        tool_name: tool_name.to_string(),
        success: true,
        output: output.to_string(),
        error: None,
    }
}

fn tool_error(tool_name: &str, msg: impl Into<String>) -> ToolResult {
    ToolResult {
        tool_name: tool_name.to_string(),
        success: false,
        output: String::new(),
        error: Some(msg.into()),
    }
}

fn read_tenants() -> Vec<TenantRecord> {
    let path = tenants_path();
    if !path.exists() {
        return vec![];
    }
    let content = fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str::<Vec<TenantRecord>>(&content).unwrap_or_default()
}

fn save_tenants(rows: &[TenantRecord]) -> Result<(), String> {
    ensure_phase3_dir()?;
    let payload = serde_json::to_string_pretty(rows).map_err(|e| e.to_string())?;
    fs::write(tenants_path(), payload).map_err(|e| e.to_string())
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

fn read_sessions() -> Vec<RemoteSessionRecord> {
    let path = sessions_path();
    if !path.exists() {
        return vec![];
    }
    let content = fs::read_to_string(path).unwrap_or_default();
    content
        .lines()
        .filter_map(|line| serde_json::from_str::<RemoteSessionRecord>(line).ok())
        .collect()
}

fn save_sessions(rows: &[RemoteSessionRecord]) -> Result<(), String> {
    ensure_phase3_dir()?;
    let payload = rows
        .iter()
        .filter_map(|r| serde_json::to_string(r).ok())
        .collect::<Vec<_>>()
        .join("\n");
    let final_payload = if payload.is_empty() {
        String::new()
    } else {
        format!("{}\n", payload)
    };
    fs::write(sessions_path(), final_payload).map_err(|e| e.to_string())
}

fn read_snapshots() -> Vec<SnapshotRecord> {
    let path = snapshots_path();
    if !path.exists() {
        return vec![];
    }
    let content = fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str::<Vec<SnapshotRecord>>(&content).unwrap_or_default()
}

fn save_snapshots(rows: &[SnapshotRecord]) -> Result<(), String> {
    ensure_phase3_dir()?;
    let payload = serde_json::to_string_pretty(rows).map_err(|e| e.to_string())?;
    fs::write(snapshots_path(), payload).map_err(|e| e.to_string())
}

fn read_attestations() -> Vec<ReleaseAttestation> {
    let path = attestations_path();
    if !path.exists() {
        return vec![];
    }
    let content = fs::read_to_string(path).unwrap_or_default();
    content
        .lines()
        .filter_map(|line| serde_json::from_str::<ReleaseAttestation>(line).ok())
        .collect()
}

fn detect_authenticode_status(file_path: &Path) -> String {
    let escaped = file_path
        .to_string_lossy()
        .replace('"', "")
        .replace('\'', "''");

    let cmd = format!(
        "$ErrorActionPreference='SilentlyContinue'; $s = Get-AuthenticodeSignature -FilePath '{}'; if ($null -eq $s) {{ 'Unknown' }} else {{ $s.Status.ToString() }}",
        escaped
    );

    match std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", &cmd])
        .creation_flags(0x08000000)
        .output()
    {
        Ok(o) if o.status.success() => {
            let status = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if status.is_empty() {
                "Unknown".to_string()
            } else {
                status
            }
        }
        _ => "Unknown".to_string(),
    }
}

fn to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{:02x}", b));
    }
    out
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let data = fs::read(path).map_err(|e| e.to_string())?;
    let hash = digest(&SHA256, &data);
    Ok(to_hex(hash.as_ref()))
}

fn operational_files_for_snapshot() -> Vec<(&'static str, PathBuf)> {
    let phase2_dir = dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("nexus-lite")
        .join("phase2");

    vec![
        ("app_settings.json", AppSettings::config_path()),
        (
            "phase2_health_snapshots.jsonl",
            phase2_dir.join("health_snapshots.jsonl"),
        ),
        (
            "phase2_incident_tickets.jsonl",
            phase2_dir.join("incident_tickets.jsonl"),
        ),
        (
            "phase2_asset_inventory.json",
            phase2_dir.join("asset_inventory.json"),
        ),
        (
            "phase2_operational_runbook.md",
            phase2_dir.join("operational_runbook.md"),
        ),
        ("phase3_tenants.json", tenants_path()),
        ("phase3_remote_sessions.jsonl", sessions_path()),
    ]
}

fn estimate_driver_issue_count(drivers_output: &str) -> usize {
    let normalized = drivers_output.to_lowercase();
    if normalized.contains("no se detectaron controladores con problemas") {
        return 0;
    }
    let numbered = drivers_output
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            let mut chars = trimmed.chars();
            matches!(chars.next(), Some(c) if c.is_ascii_digit()) && trimmed.contains(". ")
        })
        .count();

    if numbered > 0 {
        numbered
    } else if !drivers_output.trim().is_empty() {
        1
    } else {
        0
    }
}

pub fn register_tenant_endpoint(args: &serde_json::Value) -> ToolResult {
    if let Err(e) = ensure_phase3_dir() {
        return tool_error("register_tenant_endpoint", e);
    }

    let tenant_name = args["tenant_name"]
        .as_str()
        .unwrap_or("Tenant Demo")
        .trim()
        .to_string();
    let tenant_id_arg = args["tenant_id"].as_str().unwrap_or("").trim().to_string();
    let site = args["site"].as_str().unwrap_or("HQ").trim().to_string();
    let endpoint_id = args["endpoint_id"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| format!("EP-{}", Uuid::new_v4().simple()));

    let hostname = args["hostname"]
        .as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            hostname::get()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        });

    let os = args["os"]
        .as_str()
        .unwrap_or(std::env::consts::OS)
        .trim()
        .to_string();

    let mut tenants = read_tenants();

    let existing_index = if !tenant_id_arg.is_empty() {
        tenants.iter().position(|t| t.id == tenant_id_arg)
    } else {
        tenants
            .iter()
            .position(|t| t.name.eq_ignore_ascii_case(&tenant_name))
    };

    let tenant_id = if let Some(idx) = existing_index {
        let tenant = &mut tenants[idx];
        if let Some(ep) = tenant.endpoints.iter_mut().find(|e| e.id == endpoint_id) {
            ep.hostname = hostname.clone();
            ep.os = os.clone();
            ep.site = site.clone();
            ep.last_seen = Utc::now();
        } else {
            tenant.endpoints.push(EndpointRecord {
                id: endpoint_id.clone(),
                hostname: hostname.clone(),
                os: os.clone(),
                site: site.clone(),
                last_seen: Utc::now(),
            });
        }
        tenant.id.clone()
    } else {
        let new_tenant_id = if tenant_id_arg.is_empty() {
            format!("TEN-{}", Uuid::new_v4().simple())
        } else {
            tenant_id_arg
        };

        tenants.push(TenantRecord {
            id: new_tenant_id.clone(),
            name: tenant_name,
            created_at: Utc::now(),
            endpoints: vec![EndpointRecord {
                id: endpoint_id.clone(),
                hostname,
                os,
                site,
                last_seen: Utc::now(),
            }],
        });

        new_tenant_id
    };

    if let Err(e) = save_tenants(&tenants) {
        return tool_error("register_tenant_endpoint", e);
    }

    tool_success(
        "register_tenant_endpoint",
        json!({
            "tenant_id": tenant_id,
            "endpoint_id": endpoint_id,
            "total_tenants": tenants.len()
        }),
    )
}

pub fn cloud_multi_tenant_overview() -> ToolResult {
    let tenants = read_tenants();
    let endpoint_count: usize = tenants.iter().map(|t| t.endpoints.len()).sum();

    let health = phase2::health_overview();
    let health_json =
        serde_json::from_str::<serde_json::Value>(&health.output).unwrap_or_else(|_| json!({}));

    let overview = json!({
        "tenants": tenants.iter().map(|t| json!({
            "tenant_id": t.id,
            "name": t.name,
            "endpoints": t.endpoints.len(),
            "sites": t.endpoints.iter().map(|e| e.site.clone()).collect::<std::collections::BTreeSet<_>>()
        })).collect::<Vec<_>>(),
        "summary": {
            "tenant_count": tenants.len(),
            "endpoint_count": endpoint_count,
            "last_health": health_json
        }
    });

    tool_success("cloud_multi_tenant_overview", overview)
}

pub fn run_multiagent_diagnosis(args: &serde_json::Value) -> ToolResult {
    let create_ticket_on_critical = args["create_ticket_on_critical"].as_bool().unwrap_or(true);

    let health_raw = phase2::health_overview();
    let health =
        serde_json::from_str::<serde_json::Value>(&health_raw.output).unwrap_or_else(|_| json!({}));
    let risk = health["risk"].as_str().unwrap_or("bajo");

    let process_raw = processes::list_processes("cpu", 10);
    let processes_json = serde_json::from_str::<serde_json::Value>(&process_raw.output)
        .unwrap_or_else(|_| json!([]));

    let network_raw = network_diagnostic::run_network_diagnostic();

    let drivers_raw = crate::tools::drivers::list_driver_issues();
    let driver_issue_count = estimate_driver_issue_count(&drivers_raw.output);

    let tickets_raw = phase2::list_incident_tickets();
    let tickets =
        serde_json::from_str::<Vec<serde_json::Value>>(&tickets_raw.output).unwrap_or_default();

    let mut severity = "low";
    let mut actions: Vec<&str> = vec![];

    if risk == "alto" {
        severity = "critical";
        actions.push("run_automation_cycle (execute_actions=true)");
        actions.push("create_rollback_snapshot");
    } else if risk == "medio" {
        severity = "medium";
        actions.push("run_automation_cycle (execute_actions=false)");
    }

    if tickets.len() > 10 {
        actions.push("start_remote_support_session");
    }

    if driver_issue_count > 0 {
        actions.push("update_problem_drivers");
        if severity == "low" {
            severity = "medium";
        }
    }

    if network_raw.output.contains("Fallo") && severity == "low" {
        severity = "medium";
        actions.push("run_network_diagnostic");
    }

    let mut ticket_created = serde_json::Value::Null;
    if create_ticket_on_critical && severity == "critical" {
        let created = phase2::create_incident_ticket(&json!({
            "title": "Diagnóstico multiagente detectó condición crítica",
            "category": "multiagent",
            "severity": "high",
            "details": format!("Riesgo={} TicketsAbiertos={}", risk, tickets.len()),
            "source": "phase3-multiagent"
        }));
        ticket_created = serde_json::from_str::<serde_json::Value>(&created.output)
            .unwrap_or_else(|_| json!({"raw": created.output}));
    }

    let output = json!({
        "severity": severity,
        "agents": {
            "network": {
                "status": if network_raw.success { "ok" } else { "warn" },
                "details": network_raw.output
            },
            "windows": {
                "top_processes": processes_json,
                "driver_issues": {
                    "count": driver_issue_count,
                    "raw": drivers_raw.output
                }
            },
            "security": {
                "audit_events_recent": crate::tools::audit::read_audit_logs(10).len(),
                "risk": risk
            },
            "performance": {
                "health": health
            },
            "helpdesk": {
                "open_tickets": tickets.len(),
                "ticket_created": ticket_created
            }
        },
        "recommended_actions": actions
    });

    tool_success("run_multiagent_diagnosis", output)
}

pub fn start_remote_support_session(args: &serde_json::Value) -> ToolResult {
    if let Err(e) = ensure_phase3_dir() {
        return tool_error("start_remote_support_session", e);
    }

    let session_id = format!("RS-{}", Uuid::new_v4().simple());
    let tenant_id = args["tenant_id"]
        .as_str()
        .unwrap_or("TEN-LOCAL")
        .to_string();
    let endpoint_id = args["endpoint_id"]
        .as_str()
        .unwrap_or("EP-LOCAL")
        .to_string();
    let operator = args["operator"].as_str().unwrap_or("kernel-ia").to_string();
    let reason = args["reason"]
        .as_str()
        .unwrap_or("Incidente operativo")
        .to_string();
    let transport = args["transport"].as_str().unwrap_or("rustdesk").to_string();

    let connect_uri = format!(
        "{}://support/{}/{}?session={}",
        transport, tenant_id, endpoint_id, session_id
    );
    let evidence_path = phase3_dir()
        .join("remote_evidence")
        .join(format!("{}.json", session_id));

    if let Some(parent) = evidence_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return tool_error("start_remote_support_session", e.to_string());
        }
    }

    let record = RemoteSessionRecord {
        session_id: session_id.clone(),
        tenant_id,
        endpoint_id,
        operator,
        reason,
        transport,
        connect_uri: connect_uri.clone(),
        started_at: Utc::now(),
        closed_at: None,
        status: "active".to_string(),
        outcome: None,
        evidence_path: evidence_path.to_string_lossy().to_string(),
    };

    if let Err(e) = append_jsonl(&sessions_path(), &record) {
        return tool_error("start_remote_support_session", e);
    }

    let evidence = json!({
        "event": "session_started",
        "session_id": session_id,
        "timestamp": Utc::now(),
        "connect_uri": connect_uri
    });

    if let Err(e) = fs::write(
        &evidence_path,
        serde_json::to_string_pretty(&evidence).unwrap_or_else(|_| "{}".to_string()),
    ) {
        return tool_error("start_remote_support_session", e.to_string());
    }

    tool_success("start_remote_support_session", evidence)
}

pub fn close_remote_support_session(args: &serde_json::Value) -> ToolResult {
    let session_id = args["session_id"].as_str().unwrap_or("").trim().to_string();
    if session_id.is_empty() {
        return tool_error(
            "close_remote_support_session",
            "Debes enviar 'session_id' para cerrar la sesión.",
        );
    }

    let outcome = args["outcome"]
        .as_str()
        .unwrap_or("Sesión finalizada por operador")
        .to_string();

    let mut sessions = read_sessions();
    let Some(found) = sessions.iter_mut().find(|s| s.session_id == session_id) else {
        return tool_error(
            "close_remote_support_session",
            format!("No existe la sesión '{}'.", session_id),
        );
    };

    found.closed_at = Some(Utc::now());
    found.status = "closed".to_string();
    found.outcome = Some(outcome.clone());

    if let Err(e) = save_sessions(&sessions) {
        return tool_error("close_remote_support_session", e);
    }

    tool_success(
        "close_remote_support_session",
        json!({
            "session_id": session_id,
            "status": "closed",
            "outcome": outcome
        }),
    )
}

pub fn list_remote_support_sessions() -> ToolResult {
    let mut sessions = read_sessions();
    sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    tool_success(
        "list_remote_support_sessions",
        serde_json::to_value(sessions).unwrap_or_else(|_| json!([])),
    )
}

pub fn create_rollback_snapshot(args: &serde_json::Value) -> ToolResult {
    if let Err(e) = ensure_phase3_dir() {
        return tool_error("create_rollback_snapshot", e);
    }

    if let Err(e) = fs::create_dir_all(snapshots_dir()) {
        return tool_error("create_rollback_snapshot", e.to_string());
    }

    let snapshot_id = format!("SNAP-{}", Uuid::new_v4().simple());
    let snapshot_dir = snapshots_dir().join(&snapshot_id);
    if let Err(e) = fs::create_dir_all(&snapshot_dir) {
        return tool_error("create_rollback_snapshot", e.to_string());
    }

    let reason = args["reason"]
        .as_str()
        .unwrap_or("Snapshot manual antes de cambios sensibles")
        .to_string();

    let mut files_backed_up = Vec::new();

    for (backup_name, source_path) in operational_files_for_snapshot() {
        if !source_path.exists() {
            continue;
        }
        let destination = snapshot_dir.join(backup_name);
        if fs::copy(&source_path, &destination).is_ok() {
            files_backed_up.push(source_path.to_string_lossy().to_string());
        }
    }

    let mut snapshots = read_snapshots();
    snapshots.push(SnapshotRecord {
        id: snapshot_id.clone(),
        created_at: Utc::now(),
        reason,
        snapshot_dir: snapshot_dir.to_string_lossy().to_string(),
        files_backed_up: files_backed_up.clone(),
    });

    if let Err(e) = save_snapshots(&snapshots) {
        return tool_error("create_rollback_snapshot", e);
    }

    tool_success(
        "create_rollback_snapshot",
        json!({
            "snapshot_id": snapshot_id,
            "files_backed_up": files_backed_up.len(),
            "snapshot_dir": snapshot_dir
        }),
    )
}

pub fn list_rollback_snapshots() -> ToolResult {
    let mut snapshots = read_snapshots();
    snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    tool_success(
        "list_rollback_snapshots",
        serde_json::to_value(snapshots).unwrap_or_else(|_| json!([])),
    )
}

pub fn rollback_to_snapshot(args: &serde_json::Value) -> ToolResult {
    let snapshot_id = args["snapshot_id"]
        .as_str()
        .unwrap_or("")
        .trim()
        .to_string();
    if snapshot_id.is_empty() {
        return tool_error(
            "rollback_to_snapshot",
            "Debes enviar 'snapshot_id' para ejecutar rollback.",
        );
    }

    let snapshots = read_snapshots();
    let Some(snapshot) = snapshots.into_iter().find(|s| s.id == snapshot_id) else {
        return tool_error(
            "rollback_to_snapshot",
            format!("No se encontró snapshot '{}'.", snapshot_id),
        );
    };

    let snapshot_dir = PathBuf::from(&snapshot.snapshot_dir);
    if !snapshot_dir.exists() {
        return tool_error(
            "rollback_to_snapshot",
            format!(
                "El directorio del snapshot no existe: {}",
                snapshot.snapshot_dir
            ),
        );
    }

    let mut restored = Vec::new();
    for (backup_name, target_path) in operational_files_for_snapshot() {
        let backup_file = snapshot_dir.join(backup_name);
        if !backup_file.exists() {
            continue;
        }
        if let Some(parent) = target_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                return tool_error("rollback_to_snapshot", e.to_string());
            }
        }
        if fs::copy(&backup_file, &target_path).is_ok() {
            restored.push(target_path.to_string_lossy().to_string());
        }
    }

    tool_success(
        "rollback_to_snapshot",
        json!({
            "snapshot_id": snapshot_id,
            "restored_files": restored,
            "restored_count": restored.len()
        }),
    )
}

pub fn attest_release_artifact(args: &serde_json::Value) -> ToolResult {
    if let Err(e) = ensure_phase3_dir() {
        return tool_error("attest_release_artifact", e);
    }

    let file_path = args["file_path"].as_str().unwrap_or("").trim().to_string();
    let target = if file_path.is_empty() {
        std::env::current_exe().unwrap_or_else(|_| PathBuf::from(""))
    } else {
        PathBuf::from(file_path)
    };

    if !target.exists() {
        return tool_error(
            "attest_release_artifact",
            format!("No existe el archivo: {}", target.display()),
        );
    }

    let sha = match sha256_file(&target) {
        Ok(v) => v,
        Err(e) => return tool_error("attest_release_artifact", e),
    };

    let authenticode_status = detect_authenticode_status(&target);
    let reputation = match authenticode_status.as_str() {
        "Valid" => "trusted",
        "NotSigned" => "untrusted",
        _ => "unknown",
    };

    let record = ReleaseAttestation {
        id: format!("ATT-{}", Uuid::new_v4().simple()),
        file_path: target.to_string_lossy().to_string(),
        sha256: sha,
        authenticode_status,
        reputation: reputation.to_string(),
        signed_at: Utc::now(),
        signed_by: whoami::username(),
    };

    if let Err(e) = append_jsonl(&attestations_path(), &record) {
        return tool_error("attest_release_artifact", e);
    }

    tool_success(
        "attest_release_artifact",
        serde_json::to_value(record).unwrap_or_else(|_| json!({})),
    )
}

pub fn verify_release_attestation(args: &serde_json::Value) -> ToolResult {
    let file_path = args["file_path"].as_str().unwrap_or("").trim().to_string();
    let target = if file_path.is_empty() {
        std::env::current_exe().unwrap_or_else(|_| PathBuf::from(""))
    } else {
        PathBuf::from(file_path)
    };

    if !target.exists() {
        return tool_error(
            "verify_release_attestation",
            format!("No existe el archivo: {}", target.display()),
        );
    }

    let current_sha = match sha256_file(&target) {
        Ok(v) => v,
        Err(e) => return tool_error("verify_release_attestation", e),
    };

    let target_str = target.to_string_lossy().to_string();
    let latest = read_attestations()
        .into_iter()
        .filter(|r| r.file_path.eq_ignore_ascii_case(&target_str))
        .max_by(|a, b| a.signed_at.cmp(&b.signed_at));

    let Some(latest) = latest else {
        return tool_success(
            "verify_release_attestation",
            json!({
                "verified": false,
                "reason": "No hay attestations registradas para este archivo.",
                "file_path": target_str,
                "current_sha256": current_sha
            }),
        );
    };

    let verified = latest.sha256 == current_sha;
    tool_success(
        "verify_release_attestation",
        json!({
            "verified": verified,
            "file_path": target_str,
            "current_sha256": current_sha,
            "expected_sha256": latest.sha256,
            "authenticode_status": latest.authenticode_status,
            "reputation": latest.reputation,
            "signed_at": latest.signed_at
        }),
    )
}

pub fn run_phase3_smoke() -> ToolResult {
    let mut steps = Vec::new();

    let registration = register_tenant_endpoint(&json!({
        "tenant_name": "Tenant Smoke",
        "site": "LAB",
        "hostname": "nexus-smoke-host"
    }));

    let (tenant_id, endpoint_id) = if registration.success {
        let parsed = serde_json::from_str::<serde_json::Value>(&registration.output)
            .unwrap_or_else(|_| json!({}));
        (
            parsed["tenant_id"]
                .as_str()
                .unwrap_or("TEN-LOCAL")
                .to_string(),
            parsed["endpoint_id"]
                .as_str()
                .unwrap_or("EP-LOCAL")
                .to_string(),
        )
    } else {
        ("TEN-LOCAL".to_string(), "EP-LOCAL".to_string())
    };

    steps.push(json!({
        "step": "register_tenant_endpoint",
        "success": registration.success,
        "error": registration.error,
        "output": registration.output
    }));

    let overview = cloud_multi_tenant_overview();
    steps.push(json!({
        "step": "cloud_multi_tenant_overview",
        "success": overview.success,
        "error": overview.error,
        "output": overview.output
    }));

    let multiagent = run_multiagent_diagnosis(&json!({ "create_ticket_on_critical": true }));
    steps.push(json!({
        "step": "run_multiagent_diagnosis",
        "success": multiagent.success,
        "error": multiagent.error,
        "output": multiagent.output
    }));

    let start_session = start_remote_support_session(&json!({
        "tenant_id": tenant_id,
        "endpoint_id": endpoint_id,
        "operator": "phase3-smoke",
        "reason": "Validación E2E Fase 3",
        "transport": "rustdesk"
    }));

    let session_id = if start_session.success {
        serde_json::from_str::<serde_json::Value>(&start_session.output)
            .ok()
            .and_then(|v| v["session_id"].as_str().map(|s| s.to_string()))
    } else {
        None
    };

    steps.push(json!({
        "step": "start_remote_support_session",
        "success": start_session.success,
        "error": start_session.error,
        "output": start_session.output
    }));

    let session_list = list_remote_support_sessions();
    steps.push(json!({
        "step": "list_remote_support_sessions",
        "success": session_list.success,
        "error": session_list.error,
        "output": session_list.output
    }));

    if let Some(id) = session_id {
        let close = close_remote_support_session(&json!({
            "session_id": id,
            "outcome": "Smoke test completado"
        }));
        steps.push(json!({
            "step": "close_remote_support_session",
            "success": close.success,
            "error": close.error,
            "output": close.output
        }));
    }

    let snapshot = create_rollback_snapshot(&json!({ "reason": "Smoke test fase 3" }));
    let snapshot_id = if snapshot.success {
        serde_json::from_str::<serde_json::Value>(&snapshot.output)
            .ok()
            .and_then(|v| v["snapshot_id"].as_str().map(|s| s.to_string()))
    } else {
        None
    };
    steps.push(json!({
        "step": "create_rollback_snapshot",
        "success": snapshot.success,
        "error": snapshot.error,
        "output": snapshot.output
    }));

    let snapshot_list = list_rollback_snapshots();
    steps.push(json!({
        "step": "list_rollback_snapshots",
        "success": snapshot_list.success,
        "error": snapshot_list.error,
        "output": snapshot_list.output
    }));

    if let Some(id) = snapshot_id {
        let rollback = rollback_to_snapshot(&json!({ "snapshot_id": id }));
        steps.push(json!({
            "step": "rollback_to_snapshot",
            "success": rollback.success,
            "error": rollback.error,
            "output": rollback.output
        }));
    }

    let attest = attest_release_artifact(&json!({}));
    steps.push(json!({
        "step": "attest_release_artifact",
        "success": attest.success,
        "error": attest.error,
        "output": attest.output
    }));

    let verify = verify_release_attestation(&json!({}));
    steps.push(json!({
        "step": "verify_release_attestation",
        "success": verify.success,
        "error": verify.error,
        "output": verify.output
    }));

    let ok = steps
        .iter()
        .all(|s| s.get("success").and_then(|v| v.as_bool()).unwrap_or(false));

    ToolResult {
        tool_name: "run_phase3_smoke".to_string(),
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
            Some("Uno o más pasos del smoke E2E de Fase 3 fallaron.".to_string())
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase3_smoke_returns_steps_payload() {
        let result = run_phase3_smoke();
        let payload = serde_json::from_str::<serde_json::Value>(&result.output)
            .expect("run_phase3_smoke must return JSON payload");
        let steps = payload["steps"]
            .as_array()
            .expect("steps should be an array");
        assert!(!steps.is_empty(), "smoke should execute at least one step");
    }

    #[test]
    fn tenant_registration_creates_valid_payload() {
        let result = register_tenant_endpoint(&json!({
            "tenant_name": "Tenant Unit Test",
            "site": "TEST"
        }));
        assert!(result.success, "tenant registration should succeed");

        let payload = serde_json::from_str::<serde_json::Value>(&result.output)
            .expect("register_tenant_endpoint must return JSON payload");
        assert!(payload["tenant_id"].as_str().is_some());
        assert!(payload["endpoint_id"].as_str().is_some());
    }

    #[test]
    fn driver_issue_counter_handles_plain_text() {
        let input = "Controladores/dispositivos con problemas detectados:\n\n1. Audio Device\n2. Network Device";
        assert_eq!(estimate_driver_issue_count(input), 2);
        assert_eq!(
            estimate_driver_issue_count(
                "No se detectaron controladores con problemas (ConfigManagerErrorCode != 0)."
            ),
            0
        );
    }
}
