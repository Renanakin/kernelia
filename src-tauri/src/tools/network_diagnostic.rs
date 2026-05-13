use crate::tools::ToolResult;
use serde_json::{json, Value};
use std::net::{TcpStream, ToSocketAddrs, UdpSocket};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::process::Command;
use std::time::{Duration, Instant};

const TCP_TIMEOUT: Duration = Duration::from_secs(2);
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub async fn get_public_ip() -> ToolResult {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(4))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return ToolResult {
                tool_name: "get_public_ip".into(),
                success: false,
                output: String::new(),
                error: Some(format!("Error creando cliente HTTP: {}", e)),
            }
        }
    };

    let providers = [
        "https://api.ipify.org?format=json",
        "https://ifconfig.me/all.json",
    ];

    for url in providers {
        let response = client.get(url).send().await;
        if let Ok(resp) = response {
            let parsed = resp.json::<serde_json::Value>().await;
            if let Ok(value) = parsed {
                let ip = value
                    .get("ip")
                    .and_then(|v| v.as_str())
                    .or_else(|| value.get("ip_addr").and_then(|v| v.as_str()));

                if let Some(ip_addr) = ip {
                    return ToolResult {
                        tool_name: "get_public_ip".into(),
                        success: true,
                        output: json!({
                            "public_ip": ip_addr,
                            "provider": url
                        })
                        .to_string(),
                        error: None,
                    };
                }
            }
        }
    }

    ToolResult {
        tool_name: "get_public_ip".into(),
        success: false,
        output: String::new(),
        error: Some(
            "No fue posible obtener la IP publica desde proveedores disponibles.".to_string(),
        ),
    }
}

fn endpoint_check(name: &str, host: &str, port: u16) -> Value {
    let started = Instant::now();
    let dns_started = Instant::now();
    let addrs = match (host, port).to_socket_addrs() {
        Ok(addrs) => addrs.collect::<Vec<_>>(),
        Err(e) => {
            return json!({
                "name": name,
                "host": host,
                "port": port,
                "success": false,
                "latency_ms": Value::Null,
                "dns_ms": dns_started.elapsed().as_millis(),
                "total_ms": started.elapsed().as_millis(),
                "resolved_ips": [],
                "error": format!("dns_error: {}", e),
            });
        }
    };

    let mut resolved_ips: Vec<String> = Vec::new();
    for addr in &addrs {
        let ip = addr.ip().to_string();
        if !resolved_ips.contains(&ip) {
            resolved_ips.push(ip);
        }
    }

    let dns_ms = dns_started.elapsed().as_millis();
    for addr in &addrs {
        let connect_started = Instant::now();
        if TcpStream::connect_timeout(addr, TCP_TIMEOUT).is_ok() {
            return json!({
                "name": name,
                "host": host,
                "port": port,
                "success": true,
                "latency_ms": connect_started.elapsed().as_millis(),
                "dns_ms": dns_ms,
                "total_ms": started.elapsed().as_millis(),
                "resolved_ips": resolved_ips,
                "error": Value::Null,
            });
        }
    }

    json!({
        "name": name,
        "host": host,
        "port": port,
        "success": false,
        "latency_ms": Value::Null,
        "dns_ms": dns_ms,
        "total_ms": started.elapsed().as_millis(),
        "resolved_ips": resolved_ips,
        "error": "timeout",
    })
}

fn latency_label(check: &Value) -> String {
    check
        .get("latency_ms")
        .and_then(|v| v.as_u64())
        .map(|ms| format!("{} ms", ms))
        .or_else(|| {
            check
                .get("error")
                .and_then(|v| v.as_str())
                .map(|e| e.to_string())
        })
        .unwrap_or_else(|| "sin dato".to_string())
}

fn local_ip_to_internet() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

#[cfg(windows)]
fn powershell_json(script: &str) -> Value {
    let mut command = Command::new("powershell.exe");
    command
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .creation_flags(CREATE_NO_WINDOW);
    let output = command.output();

    let Ok(output) = output else {
        return Value::Null;
    };
    if !output.status.success() {
        return Value::Null;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        return Value::Null;
    }

    serde_json::from_str(&stdout).unwrap_or_else(|_| json!({ "raw": stdout }))
}

#[cfg(not(windows))]
fn powershell_json(_script: &str) -> Value {
    Value::Null
}

fn network_snapshot() -> Value {
    let default_route = powershell_json(
        "Get-NetRoute -DestinationPrefix '0.0.0.0/0' -ErrorAction SilentlyContinue | Sort-Object RouteMetric | Select-Object -First 1 ifIndex,InterfaceAlias,NextHop,RouteMetric | ConvertTo-Json -Compress",
    );
    let active_adapters = powershell_json(
        "Get-NetAdapter -ErrorAction SilentlyContinue | Where-Object {$_.Status -eq 'Up'} | Select-Object -First 6 Name,InterfaceDescription,Status,LinkSpeed,MacAddress | ConvertTo-Json -Compress",
    );
    let dns_servers = powershell_json(
        "Get-DnsClientServerAddress -AddressFamily IPv4 -ErrorAction SilentlyContinue | Where-Object {$_.ServerAddresses.Count -gt 0} | Select-Object -First 6 InterfaceAlias,ServerAddresses | ConvertTo-Json -Compress",
    );

    json!({
        "local_ip_to_internet": local_ip_to_internet(),
        "default_route": default_route,
        "active_adapters": active_adapters,
        "dns_servers": dns_servers,
    })
}

fn field_as_string(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(|v| match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    })
}

fn first_object(value: &Value) -> Option<&Value> {
    value
        .as_array()
        .and_then(|items| items.first())
        .or_else(|| value.as_object().map(|_| value))
}

fn route_label(snapshot: &Value) -> String {
    let route = snapshot
        .get("default_route")
        .and_then(first_object)
        .unwrap_or(&Value::Null);
    let gateway = field_as_string(route, "NextHop").unwrap_or_else(|| "sin dato".to_string());
    let interface =
        field_as_string(route, "InterfaceAlias").unwrap_or_else(|| "sin interfaz".to_string());
    let metric = field_as_string(route, "RouteMetric").unwrap_or_else(|| "-".to_string());
    format!("{} via {} (metrica {})", gateway, interface, metric)
}

fn adapter_label(snapshot: &Value) -> String {
    let adapter = snapshot
        .get("active_adapters")
        .and_then(first_object)
        .unwrap_or(&Value::Null);
    let name = field_as_string(adapter, "Name").unwrap_or_else(|| "sin dato".to_string());
    let speed = field_as_string(adapter, "LinkSpeed").unwrap_or_else(|| "sin dato".to_string());
    format!("{} ({})", name, speed)
}

fn dns_label(snapshot: &Value) -> String {
    let dns = snapshot
        .get("dns_servers")
        .and_then(first_object)
        .unwrap_or(&Value::Null);
    let interface =
        field_as_string(dns, "InterfaceAlias").unwrap_or_else(|| "sin dato".to_string());
    let servers = dns
        .get("ServerAddresses")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "sin dato".to_string());
    format!("{} -> {}", interface, servers)
}

fn collect_network_diagnostic() -> Value {
    let checks = vec![
        endpoint_check("Google DNS TCP", "8.8.8.8", 53),
        endpoint_check("Cloudflare DNS TCP", "1.1.1.1", 53),
        endpoint_check("Google HTTPS", "google.com", 443),
        endpoint_check("Microsoft HTTPS", "www.microsoft.com", 443),
        endpoint_check("GitHub HTTPS", "github.com", 443),
    ];

    let ok_count = checks
        .iter()
        .filter(|check| {
            check
                .get("success")
                .and_then(|value| value.as_bool())
                .unwrap_or(false)
        })
        .count();
    let total = checks.len();
    let status = if ok_count == total {
        "operativa"
    } else if ok_count >= 3 {
        "degradada"
    } else {
        "critica"
    };

    let google_dns = checks
        .iter()
        .find(|check| check.get("name").and_then(|v| v.as_str()) == Some("Google DNS TCP"))
        .cloned()
        .unwrap_or_else(|| endpoint_check("Google DNS TCP", "8.8.8.8", 53));
    let google_https = checks
        .iter()
        .find(|check| check.get("name").and_then(|v| v.as_str()) == Some("Google HTTPS"))
        .cloned()
        .unwrap_or_else(|| endpoint_check("Google HTTPS", "google.com", 443));

    json!({
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "status": status,
        "score": {
            "success": ok_count,
            "total": total,
            "percent": ((ok_count as f64 / total as f64) * 100.0).round() as u64,
        },
        "snapshot": network_snapshot(),
        "connectivity": {
            "google_ping": {
                "success": google_dns.get("success").cloned().unwrap_or(Value::Bool(false)),
                "latency": latency_label(&google_dns),
                "latency_ms": google_dns.get("latency_ms").cloned().unwrap_or(Value::Null),
            },
            "dns_resolution": {
                "success": google_https.get("success").cloned().unwrap_or(Value::Bool(false)),
                "latency": latency_label(&google_https),
                "resolved_ips": google_https.get("resolved_ips").cloned().unwrap_or_else(|| json!([])),
            },
            "checks": checks,
        }
    })
}

fn format_network_diagnostic(diagnostic: &Value) -> String {
    let generated_at = diagnostic
        .get("generated_at")
        .and_then(|v| v.as_str())
        .unwrap_or("sin timestamp");
    let status = diagnostic
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("sin estado");
    let score = diagnostic.get("score").unwrap_or(&Value::Null);
    let success = score.get("success").and_then(|v| v.as_u64()).unwrap_or(0);
    let total = score.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
    let percent = score.get("percent").and_then(|v| v.as_u64()).unwrap_or(0);
    let snapshot = diagnostic.get("snapshot").unwrap_or(&Value::Null);
    let local_ip = snapshot
        .get("local_ip_to_internet")
        .and_then(|v| v.as_str())
        .unwrap_or("sin dato");

    let mut lines = vec![
        format!("Diagnostico de red en tiempo real"),
        format!("Generado: {}", generated_at),
        format!(
            "Estado general: {} ({} de {} pruebas OK, {}%)",
            status, success, total, percent
        ),
        String::new(),
        "Ruta local".to_string(),
        format!("- IP local hacia internet: {}", local_ip),
        format!("- Gateway: {}", route_label(snapshot)),
        format!("- Adaptador activo principal: {}", adapter_label(snapshot)),
        format!("- DNS configurado: {}", dns_label(snapshot)),
        String::new(),
        "Pruebas TCP/DNS".to_string(),
    ];

    if let Some(checks) = diagnostic
        .pointer("/connectivity/checks")
        .and_then(|v| v.as_array())
    {
        for check in checks {
            let name = check
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("check");
            let host = check.get("host").and_then(|v| v.as_str()).unwrap_or("-");
            let port = check.get("port").and_then(|v| v.as_u64()).unwrap_or(0);
            let ok = check
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let dns_ms = check.get("dns_ms").and_then(|v| v.as_u64()).unwrap_or(0);
            let ips = check
                .get("resolved_ips")
                .and_then(|v| v.as_array())
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str())
                        .take(3)
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "sin resolucion".to_string());

            lines.push(format!(
                "- {} ({}:{}): {} | latencia {} | DNS {} ms | IPs {}",
                name,
                host,
                port,
                if ok { "OK" } else { "FALLO" },
                latency_label(check),
                dns_ms,
                ips
            ));
        }
    }

    lines.push(String::new());
    lines.push(match status {
        "operativa" => "Evaluacion: la conectividad base esta operativa. Si una app falla, revisar proxy, firewall o DNS especifico de esa app.".to_string(),
        "degradada" => "Evaluacion: hay conectividad parcial. Revisar DNS, gateway, firewall local o saturacion del enlace.".to_string(),
        _ => "Evaluacion: conectividad critica. Validar cable/WiFi, gateway, DNS y salida a internet.".to_string(),
    });

    lines.join("\n")
}

pub fn run_network_diagnostic_json() -> String {
    collect_network_diagnostic().to_string()
}

pub fn run_network_diagnostic() -> ToolResult {
    let diagnostic = collect_network_diagnostic();

    ToolResult {
        tool_name: "run_network_diagnostic".into(),
        success: true,
        output: format_network_diagnostic(&diagnostic),
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{format_network_diagnostic, run_network_diagnostic_json};

    #[test]
    fn diagnostic_json_keeps_telemetry_contract() {
        let raw = run_network_diagnostic_json();
        let parsed: serde_json::Value = serde_json::from_str(&raw).expect("valid json");

        assert!(parsed.get("generated_at").is_some());
        assert!(parsed
            .pointer("/connectivity/google_ping/success")
            .is_some());
        assert!(parsed
            .pointer("/connectivity/google_ping/latency")
            .is_some());
        assert!(parsed
            .pointer("/connectivity/dns_resolution/success")
            .is_some());
        assert!(parsed.pointer("/connectivity/checks").is_some());
    }

    #[test]
    fn diagnostic_report_is_not_static_two_line_template() {
        let raw = run_network_diagnostic_json();
        let parsed: serde_json::Value = serde_json::from_str(&raw).expect("valid json");
        let report = format_network_diagnostic(&parsed);

        assert!(report.contains("Generado:"));
        assert!(report.contains("Ruta local"));
        assert!(report.contains("Pruebas TCP/DNS"));
        assert!(report.contains("GitHub HTTPS"));
    }
}
