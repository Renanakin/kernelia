use crate::tools::{audit, ToolResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HealthSnapshot {
    timestamp: DateTime<Utc>,
    cpu_usage: f32,
    memory_pct: f32,
    disk_pct: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IncidentTicket {
    id: String,
    timestamp: DateTime<Utc>,
    title: String,
    category: String,
    severity: String,
    details: String,
    source: String,
    status: String,
}

fn phase2_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("nexus-lite")
        .join("phase2")
}

fn health_log_path() -> PathBuf {
    phase2_dir().join("health_snapshots.jsonl")
}

fn tickets_log_path() -> PathBuf {
    phase2_dir().join("incident_tickets.jsonl")
}

fn inventory_path() -> PathBuf {
    phase2_dir().join("asset_inventory.json")
}

fn docs_path() -> PathBuf {
    phase2_dir().join("operational_runbook.md")
}

fn ensure_phase2_dir() -> Result<(), String> {
    fs::create_dir_all(phase2_dir()).map_err(|e| e.to_string())
}

fn run_powershell_json(script: &str) -> serde_json::Value {
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", script])
        .creation_flags(0x08000000)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let txt = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if txt.is_empty() {
                json!({})
            } else {
                serde_json::from_str(&txt).unwrap_or_else(|_| json!({ "raw": txt }))
            }
        }
        Ok(o) => json!({
            "error": String::from_utf8_lossy(&o.stderr).trim().to_string()
        }),
        Err(e) => json!({ "error": e.to_string() }),
    }
}

fn get_current_metrics() -> Result<HealthSnapshot, String> {
    let raw = crate::tools::sysinfo_tool::get_system_info_json();
    let v: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let cpu = v["cpu_usage"].as_f64().unwrap_or(0.0) as f32;
    let mem_used = v["memory_used"].as_f64().unwrap_or(0.0);
    let mem_total = v["memory_total"].as_f64().unwrap_or(1.0);
    let memory_pct = if mem_total > 0.0 {
        ((mem_used / mem_total) * 100.0) as f32
    } else {
        0.0
    };
    let mut disk_pct = 0.0f32;
    if let Some(disks) = v["disks"].as_array() {
        if let Some(main) = disks.first() {
            let used = main["used_space"].as_f64().unwrap_or(0.0);
            let total = main["total_space"].as_f64().unwrap_or(1.0);
            disk_pct = if total > 0.0 {
                ((used / total) * 100.0) as f32
            } else {
                0.0
            };
        }
    }
    Ok(HealthSnapshot {
        timestamp: Utc::now(),
        cpu_usage: cpu,
        memory_pct,
        disk_pct,
    })
}

fn append_health_snapshot(s: &HealthSnapshot) -> Result<(), String> {
    ensure_phase2_dir()?;
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(health_log_path())
        .map_err(|e| e.to_string())?;
    let line = serde_json::to_string(s).map_err(|e| e.to_string())? + "\n";
    f.write_all(line.as_bytes()).map_err(|e| e.to_string())
}

fn read_health_snapshots(limit: usize) -> Vec<HealthSnapshot> {
    let p = health_log_path();
    if !p.exists() {
        return vec![];
    }
    let content = fs::read_to_string(p).unwrap_or_default();
    let mut rows: Vec<HealthSnapshot> = content
        .lines()
        .filter_map(|l| serde_json::from_str::<HealthSnapshot>(l).ok())
        .collect();
    rows.reverse();
    rows.into_iter().take(limit).collect()
}

pub fn health_overview() -> ToolResult {
    let snapshot = match get_current_metrics() {
        Ok(s) => s,
        Err(e) => {
            return ToolResult {
                tool_name: "health_overview".into(),
                success: false,
                output: String::new(),
                error: Some(e),
            }
        }
    };
    let _ = append_health_snapshot(&snapshot);
    let history = read_health_snapshots(30);

    let score_cpu = (100.0 - snapshot.cpu_usage).clamp(0.0, 100.0);
    let score_mem = (100.0 - snapshot.memory_pct).clamp(0.0, 100.0);
    let score_disk = (100.0 - snapshot.disk_pct).clamp(0.0, 100.0);
    let health_score =
        ((score_cpu * 0.35) + (score_mem * 0.35) + (score_disk * 0.30)).round() as i32;

    let disk_trend = if history.len() > 1 {
        let newest = history
            .first()
            .map(|h| h.disk_pct)
            .unwrap_or(snapshot.disk_pct);
        let oldest = history
            .last()
            .map(|h| h.disk_pct)
            .unwrap_or(snapshot.disk_pct);
        newest - oldest
    } else {
        0.0
    };

    let anomaly =
        snapshot.cpu_usage > 90.0 || snapshot.memory_pct > 90.0 || snapshot.disk_pct > 95.0;
    let risk = if health_score < 40 || anomaly {
        "alto"
    } else if health_score < 70 {
        "medio"
    } else {
        "bajo"
    };

    let out = json!({
        "health_score": health_score,
        "risk": risk,
        "current": {
            "cpu_pct": snapshot.cpu_usage,
            "memory_pct": snapshot.memory_pct,
            "disk_pct": snapshot.disk_pct
        },
        "trend": {
            "disk_delta_pct": disk_trend,
            "samples": history.len()
        },
        "anomaly_detected": anomaly
    });

    ToolResult {
        tool_name: "health_overview".into(),
        success: true,
        output: out.to_string(),
        error: None,
    }
}

pub fn health_summary() -> ToolResult {
    let overview = health_overview();
    if !overview.success {
        return ToolResult {
            tool_name: "health_summary".into(),
            success: false,
            output: String::new(),
            error: overview.error,
        };
    }

    let parsed: serde_json::Value =
        serde_json::from_str(&overview.output).unwrap_or_else(|_| json!({}));
    let score = parsed["health_score"].as_i64().unwrap_or(0);
    let risk = parsed["risk"].as_str().unwrap_or("desconocido");
    let cpu = parsed["current"]["cpu_pct"].as_f64().unwrap_or(0.0);
    let memory = parsed["current"]["memory_pct"].as_f64().unwrap_or(0.0);
    let disk = parsed["current"]["disk_pct"].as_f64().unwrap_or(0.0);
    let anomaly = parsed["anomaly_detected"].as_bool().unwrap_or(false);

    let status = if score >= 80 {
        "saludable"
    } else if score >= 60 {
        "estable con carga moderada"
    } else if score >= 40 {
        "degradado"
    } else {
        "critico"
    };

    let action = if cpu >= 85.0 {
        "Revisar procesos con mayor uso de CPU."
    } else if memory >= 85.0 {
        "Revisar consumo de RAM y aplicaciones abiertas."
    } else if disk >= 85.0 {
        "Liberar espacio o revisar unidades con alto uso."
    } else {
        "No se requiere accion inmediata."
    };

    let output = format!(
        "Health del equipo: {}/100 ({})\nRiesgo operacional: {}\n\nLectura actual:\n- CPU: {:.1}%\n- RAM: {:.1}%\n- Disco principal: {:.1}%\n- Anomalia detectada: {}\n\nAccion sugerida: {}",
        score,
        status,
        risk,
        cpu,
        memory,
        disk,
        if anomaly { "si" } else { "no" },
        action
    );

    ToolResult {
        tool_name: "health_summary".into(),
        success: true,
        output,
        error: None,
    }
}

pub fn scan_asset_inventory() -> ToolResult {
    if let Err(e) = ensure_phase2_dir() {
        return ToolResult {
            tool_name: "scan_asset_inventory".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    let system_raw = crate::tools::sysinfo_tool::get_system_info_json();
    let apps_cmd = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-Command",
            r#"$ErrorActionPreference='SilentlyContinue'; 
$apps = Get-ItemProperty HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\* |
  Where-Object { $_.DisplayName } |
  Select-Object -First 150 DisplayName, DisplayVersion, Publisher;
if ($null -eq $apps) { '[]' } else { $apps | ConvertTo-Json -Depth 3 -Compress }"#,
        ])
        .creation_flags(0x08000000)
        .output();

    let installed_apps = match apps_cmd {
        Ok(o) if o.status.success() => {
            let txt = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if txt.is_empty() {
                json!([])
            } else {
                serde_json::from_str::<serde_json::Value>(&txt).unwrap_or_else(|_| json!([]))
            }
        }
        _ => json!([]),
    };

    let inventory = json!({
        "scanned_at": Utc::now(),
        "system": serde_json::from_str::<serde_json::Value>(&system_raw).unwrap_or_else(|_| json!({})),
        "installed_apps": installed_apps
    });

    match fs::write(
        inventory_path(),
        serde_json::to_string_pretty(&inventory).unwrap_or_else(|_| "{}".into()),
    ) {
        Ok(_) => ToolResult {
            tool_name: "scan_asset_inventory".into(),
            success: true,
            output: inventory.to_string(),
            error: None,
        },
        Err(e) => ToolResult {
            tool_name: "scan_asset_inventory".into(),
            success: false,
            output: String::new(),
            error: Some(e.to_string()),
        },
    }
}

pub fn create_incident_ticket(args: &serde_json::Value) -> ToolResult {
    if let Err(e) = ensure_phase2_dir() {
        return ToolResult {
            tool_name: "create_incident_ticket".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    let ticket = IncidentTicket {
        id: format!("INC-{}", Utc::now().format("%Y%m%d-%H%M%S")),
        timestamp: Utc::now(),
        title: args["title"]
            .as_str()
            .unwrap_or("Incidente detectado por Kernel IA")
            .to_string(),
        category: args["category"].as_str().unwrap_or("general").to_string(),
        severity: args["severity"].as_str().unwrap_or("medium").to_string(),
        details: args["details"]
            .as_str()
            .unwrap_or("Sin detalles")
            .to_string(),
        source: args["source"].as_str().unwrap_or("kernel-ia").to_string(),
        status: "open".to_string(),
    };

    let mut file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(tickets_log_path())
    {
        Ok(f) => f,
        Err(e) => {
            return ToolResult {
                tool_name: "create_incident_ticket".into(),
                success: false,
                output: String::new(),
                error: Some(e.to_string()),
            }
        }
    };

    let line = serde_json::to_string(&ticket).unwrap_or_else(|_| "{}".into()) + "\n";
    if let Err(e) = file.write_all(line.as_bytes()) {
        return ToolResult {
            tool_name: "create_incident_ticket".into(),
            success: false,
            output: String::new(),
            error: Some(e.to_string()),
        };
    }

    ToolResult {
        tool_name: "create_incident_ticket".into(),
        success: true,
        output: serde_json::to_string(&ticket).unwrap_or_else(|_| "{}".into()),
        error: None,
    }
}

pub fn list_incident_tickets() -> ToolResult {
    let p = tickets_log_path();
    if !p.exists() {
        return ToolResult {
            tool_name: "list_incident_tickets".into(),
            success: true,
            output: "[]".into(),
            error: None,
        };
    }
    let content = fs::read_to_string(p).unwrap_or_default();
    let mut tickets: Vec<IncidentTicket> = content
        .lines()
        .filter_map(|l| serde_json::from_str::<IncidentTicket>(l).ok())
        .collect();
    tickets.reverse();
    ToolResult {
        tool_name: "list_incident_tickets".into(),
        success: true,
        output: serde_json::to_string(&tickets).unwrap_or_else(|_| "[]".into()),
        error: None,
    }
}

pub async fn run_automation_cycle(
    _app: &tauri::AppHandle,
    _role: crate::tools::rbac::UserRole,
    execute_actions: bool,
) -> ToolResult {
    let snapshot = match get_current_metrics() {
        Ok(s) => s,
        Err(e) => {
            return ToolResult {
                tool_name: "run_automation_cycle".into(),
                success: false,
                output: String::new(),
                error: Some(e),
            }
        }
    };
    let _ = append_health_snapshot(&snapshot);

    let mut planned_actions: Vec<&str> = Vec::new();
    if snapshot.disk_pct > 90.0 {
        planned_actions.push("run_cleanup");
    }
    if snapshot.memory_pct > 92.0 {
        planned_actions.push("list_processes");
    }
    if snapshot.cpu_usage > 95.0 {
        planned_actions.push("list_processes");
    }

    let mut executed = Vec::new();
    if execute_actions {
        for action in planned_actions.iter().copied() {
            let result = match action {
                "run_cleanup" => crate::tools::cleanup::run_cleanup(None).await,
                "list_processes" => crate::tools::processes::list_processes("memory", 10),
                _ => ToolResult {
                    tool_name: action.to_string(),
                    success: false,
                    output: String::new(),
                    error: Some("Accion no soportada".to_string()),
                },
            };
            executed.push(json!({
                "tool": action,
                "success": result.success,
                "error": result.error
            }));
        }
    }

    if snapshot.disk_pct > 95.0 || snapshot.memory_pct > 95.0 {
        let _ = create_incident_ticket(&json!({
            "title": "Riesgo operacional detectado automáticamente",
            "category": "observability",
            "severity": "high",
            "details": format!("CPU {:.1}%, RAM {:.1}%, Disco {:.1}%", snapshot.cpu_usage, snapshot.memory_pct, snapshot.disk_pct),
            "source": "automation-cycle"
        }));
    }

    ToolResult {
        tool_name: "run_automation_cycle".into(),
        success: true,
        output: json!({
            "executed_mode": execute_actions,
            "planned_actions": planned_actions,
            "executed_actions": executed,
            "metrics": {
                "cpu_pct": snapshot.cpu_usage,
                "memory_pct": snapshot.memory_pct,
                "disk_pct": snapshot.disk_pct
            }
        })
        .to_string(),
        error: None,
    }
}

pub async fn run_operational_suite(
    app: &tauri::AppHandle,
    role: crate::tools::rbac::UserRole,
    execute_maintenance: bool,
) -> ToolResult {
    let system_info = crate::tools::sysinfo_tool::get_system_info();
    let health = health_overview();
    let network = crate::tools::network_diagnostic::run_network_diagnostic();
    let public_ip = crate::tools::network_diagnostic::get_public_ip().await;
    let services = crate::tools::windows_services::list_running_services();
    let drivers = crate::tools::drivers::list_driver_issues();

    let windows_update = run_powershell_json(
        r#"$svc = Get-Service -Name wuauserv -ErrorAction SilentlyContinue;
if ($null -eq $svc) { '{"available":false}' } else { @{available=$true;status=$svc.Status.ToString();start_type=$svc.StartType.ToString()} | ConvertTo-Json -Compress }"#,
    );

    let security_surface = run_powershell_json(
        r#"$ports = Get-NetTCPConnection -State Listen -ErrorAction SilentlyContinue | Select-Object -First 50 LocalAddress,LocalPort,OwningProcess;
$fw = Get-NetFirewallProfile -ErrorAction SilentlyContinue | Select-Object Name,Enabled,DefaultInboundAction,DefaultOutboundAction;
@{listening_ports=$ports; firewall=$fw} | ConvertTo-Json -Depth 4 -Compress"#,
    );

    let hardware_health = run_powershell_json(
        r#"$disks = Get-PhysicalDisk -ErrorAction SilentlyContinue | Select-Object FriendlyName,HealthStatus,OperationalStatus,MediaType;
@{disks=$disks} | ConvertTo-Json -Depth 4 -Compress"#,
    );

    let maintenance = run_automation_cycle(app, role, execute_maintenance).await;
    let inventory = scan_asset_inventory();

    let health_json: serde_json::Value =
        serde_json::from_str(&health.output).unwrap_or_else(|_| json!({ "raw": health.output }));
    let risk = health_json
        .get("risk")
        .and_then(|v| v.as_str())
        .unwrap_or("desconocido")
        .to_string();

    if risk == "alto" {
        let _ = create_incident_ticket(&json!({
            "title": "Riesgo alto detectado en ciclo operacional Fase 2",
            "category": "operational-suite",
            "severity": "high",
            "details": format!("Se detectó riesgo '{}' en health_overview", risk),
            "source": "phase2-operational-suite"
        }));
    }

    let out = json!({
        "phase": 2,
        "executed_at": Utc::now(),
        "execute_maintenance": execute_maintenance,
        "windows": {
            "system_info_ok": system_info.success,
            "running_services_ok": services.success,
            "windows_update": windows_update
        },
        "network": {
            "diagnostic": network.output,
            "public_ip": if public_ip.success { serde_json::from_str::<serde_json::Value>(&public_ip.output).unwrap_or_else(|_| json!({"raw": public_ip.output})) } else { json!({"error": public_ip.error}) }
        },
        "hardware": {
            "drivers": drivers.output,
            "disk_health": hardware_health
        },
        "security": security_surface,
        "maintenance": {
            "automation_cycle": serde_json::from_str::<serde_json::Value>(&maintenance.output).unwrap_or_else(|_| json!({"raw": maintenance.output})),
            "inventory_ok": inventory.success
        },
        "health": health_json
    });

    ToolResult {
        tool_name: "run_operational_suite".into(),
        success: true,
        output: out.to_string(),
        error: None,
    }
}

pub fn generate_operational_documentation() -> ToolResult {
    if let Err(e) = ensure_phase2_dir() {
        return ToolResult {
            tool_name: "generate_operational_documentation".into(),
            success: false,
            output: String::new(),
            error: Some(e),
        };
    }

    let health = health_overview();
    let tickets = list_incident_tickets();
    let audits = audit::read_audit_logs(25);

    let md = format!(
        "# Runbook Operacional Kernel IA\n\nGenerado: {}\n\n## Salud del sistema\n\n```json\n{}\n```\n\n## Tickets recientes\n\n```json\n{}\n```\n\n## Auditoría reciente\n\n```json\n{}\n```\n",
        Utc::now().to_rfc3339(),
        health.output,
        tickets.output,
        serde_json::to_string_pretty(&audits).unwrap_or_else(|_| "[]".into())
    );

    match fs::write(docs_path(), &md) {
        Ok(_) => ToolResult {
            tool_name: "generate_operational_documentation".into(),
            success: true,
            output: format!(
                "Documentación operativa generada en: {}",
                docs_path().display()
            ),
            error: None,
        },
        Err(e) => ToolResult {
            tool_name: "generate_operational_documentation".into(),
            success: false,
            output: String::new(),
            error: Some(e.to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::health_overview;

    #[test]
    fn health_overview_returns_score() {
        let res = health_overview();
        assert!(res.success);
        let parsed: serde_json::Value = serde_json::from_str(&res.output).expect("valid json");
        assert!(parsed.get("health_score").is_some());
    }
}
