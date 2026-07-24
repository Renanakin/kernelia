use crate::config::AppSettings;
use crate::rag::models::RagConfig;
use crate::rag::storage::sqlite::ensure_database_ready;
use crate::tools::{ToolEngine, ToolResult};
use chrono::Utc;
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct VerificationRule {
    pub tool_name: String,
    pub verify_tool_name: String,
    pub verify_args_json: String,
    pub expected_condition: String,
}

#[derive(Debug, Clone)]
pub struct VerificationOutcome {
    pub verified: bool,
    pub skipped: bool,
    pub verify_tool_name: Option<String>,
    pub verify_args: Option<Value>,
    pub verification_result: String,
    pub reason_code: String,
}

pub async fn verify_tool_execution(
    app: &tauri::AppHandle,
    tool_name: &str,
    executed_args: &Value,
    tool_result: &ToolResult,
    settings: &AppSettings,
) -> VerificationOutcome {
    let Some(rule) = load_verification_rule(tool_name).ok().flatten() else {
        return VerificationOutcome {
            verified: false,
            skipped: true,
            verify_tool_name: None,
            verify_args: None,
            verification_result: "No hay regla de verificacion registrada.".to_string(),
            reason_code: "VERIFY_RULE_MISSING".to_string(),
        };
    };

    let verify_args = resolve_verify_args(&rule.verify_args_json, executed_args);
    let verify_tool_name = rule.verify_tool_name.clone();

    if crate::tools::rbac::is_owner_only_tool(&verify_tool_name) && !settings.is_megaboss_unlocked()
    {
        return VerificationOutcome {
            verified: false,
            skipped: true,
            verify_tool_name: Some(verify_tool_name),
            verify_args: Some(verify_args),
            verification_result: "Verificacion omitida: el verificador requiere privilegio MegaBoss."
                .to_string(),
            reason_code: "VERIFY_PRIVILEGE_BLOCKED".to_string(),
        };
    }

    let verification_tool_result =
        ToolEngine::execute(app, &rule.verify_tool_name, &verify_args, settings.user_role).await;
    let verified = evaluate_expected_condition(
        &rule.expected_condition,
        tool_name,
        executed_args,
        tool_result,
        &verification_tool_result,
    );
    let verification_result = format_verification_result(
        &rule,
        &verify_args,
        &verification_tool_result,
        verified,
    );

    let _ = persist_verification_trace(
        tool_name,
        executed_args,
        tool_result,
        &rule,
        &verify_args,
        &verification_tool_result,
        verified,
    );

    VerificationOutcome {
        verified,
        skipped: false,
        verify_tool_name: Some(rule.verify_tool_name),
        verify_args: Some(verify_args),
        verification_result,
        reason_code: if verified {
            "VERIFY_OK".to_string()
        } else {
            "VERIFY_FAILED".to_string()
        },
    }
}

fn load_verification_rule(tool_name: &str) -> Result<Option<VerificationRule>, String> {
    let conn = ensure_database_ready(&RagConfig::default())?;
    let mut stmt = conn
        .prepare(
            "SELECT tc.tool_name, tp.verify_tool_name, tp.verify_args_json, tp.expected_condition
             FROM tool_postcondition tp
             INNER JOIN tool_capability tc ON tc.id = tp.tool_id
             WHERE tc.tool_name = ?1
             LIMIT 1",
        )
        .map_err(|e| format!("No se pudo preparar lookup de verificacion: {}", e))?;

    stmt.query_row(params![tool_name], |row| {
        Ok(VerificationRule {
            tool_name: row.get(0)?,
            verify_tool_name: row.get(1)?,
            verify_args_json: row.get(2)?,
            expected_condition: row.get(3)?,
        })
    })
    .map(Some)
    .or_else(|err| {
        if matches!(err, rusqlite::Error::QueryReturnedNoRows) {
            Ok(None)
        } else {
            Err(format!("No se pudo leer regla de verificacion: {}", err))
        }
    })
}

fn resolve_verify_args(template_json: &str, executed_args: &Value) -> Value {
    let base = serde_json::from_str::<Value>(template_json).unwrap_or_else(|_| json!({}));
    substitute_placeholders(base, executed_args)
}

fn substitute_placeholders(value: Value, executed_args: &Value) -> Value {
    match value {
        Value::String(text) => Value::String(resolve_placeholder(&text, executed_args)),
        Value::Array(items) => Value::Array(
            items.into_iter()
                .map(|item| substitute_placeholders(item, executed_args))
                .collect(),
        ),
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (key, item) in map {
                out.insert(key, substitute_placeholders(item, executed_args));
            }
            Value::Object(out)
        }
        other => other,
    }
}

fn resolve_placeholder(text: &str, executed_args: &Value) -> String {
    if text == "<service_name>" {
        return executed_args
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("Spooler")
            .to_string();
    }

    if text == "<pid>" {
        return executed_args
            .get("pid")
            .and_then(Value::as_i64)
            .map(|value| value.to_string())
            .or_else(|| {
                executed_args
                    .get("pid")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .unwrap_or_default();
    }

    text.to_string()
}

fn evaluate_expected_condition(
    expected_condition: &str,
    tool_name: &str,
    executed_args: &Value,
    tool_result: &ToolResult,
    verification_tool_result: &ToolResult,
) -> bool {
    match expected_condition {
        "diagnostic_completed" => verification_tool_result.success,
        "dns_resolution_available" => {
            verification_tool_result.success
                && !contains_error_markers(&verification_tool_result.output)
                && !verification_tool_result.output.trim().is_empty()
        }
        "service_running" => {
            verification_tool_result.success
                && verification_tool_result
                    .output
                    .to_lowercase()
                    .contains("running")
        }
        "process_not_found" => {
            !verification_tool_result.success
                || verification_tool_result
                    .error
                    .as_deref()
                    .unwrap_or_default()
                    .to_lowercase()
                    .contains("no encontrado")
                || verification_tool_result.output.to_lowercase().contains("not found")
        }
        "recoverable_space_reduced" => verification_tool_result.success,
        "scan_registered" => {
            verification_tool_result.success
                && verification_tool_result.output.to_lowercase().contains("enabled")
        }
        _ => {
            let _ = (tool_name, executed_args, tool_result);
            verification_tool_result.success
        }
    }
}

fn contains_error_markers(text: &str) -> bool {
    let lowered = text.to_lowercase();
    lowered.contains("dns request timed out")
        || lowered.contains("non-existent domain")
        || lowered.contains("can't find")
        || lowered.contains("server failed")
}

fn format_verification_result(
    rule: &VerificationRule,
    verify_args: &Value,
    verification_tool_result: &ToolResult,
    verified: bool,
) -> String {
    format!(
        "tool={} verify_tool={} expected_condition={} verified={} verify_args={} result_success={} result_output={}",
        rule.tool_name,
        rule.verify_tool_name,
        rule.expected_condition,
        verified,
        verify_args,
        verification_tool_result.success,
        trim_for_log(&verification_tool_result.output)
    )
}

fn trim_for_log(text: &str) -> String {
    let single = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if single.len() > 240 {
        format!("{}...", &single[..240])
    } else {
        single
    }
}

fn persist_verification_trace(
    tool_name: &str,
    executed_args: &Value,
    tool_result: &ToolResult,
    rule: &VerificationRule,
    verify_args: &Value,
    verification_tool_result: &ToolResult,
    verified: bool,
) -> Result<(), String> {
    let conn = ensure_database_ready(&RagConfig::default())?;
    let trace_id = format!("trace_verify_{}", Uuid::new_v4().simple());
    let now = Utc::now().to_rfc3339();
    let tool_trace_id = format!("ttc_{}", Uuid::new_v4().simple());
    let verification_id = format!("tver_{}", Uuid::new_v4().simple());

    insert_trace_request(&conn, &trace_id, tool_name, &now)?;

    conn.execute(
        "INSERT INTO trace_tool_call (
            id, trace_id, tool_name, args_json, result_status, verification_status
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            tool_trace_id,
            trace_id,
            tool_name,
            executed_args.to_string(),
            if tool_result.success { "success" } else { "failed" },
            if verified { "verified" } else { "failed" }
        ],
    )
    .map_err(|e| format!("No se pudo insertar trace_tool_call: {}", e))?;

    conn.execute(
        "INSERT INTO trace_verification (
            id, trace_id, tool_name, verification_tool_name, verification_result, verified
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            verification_id,
            trace_id,
            tool_name,
            rule.verify_tool_name,
            format!(
                "expected={} verify_args={} result_success={} verifier_output={}",
                rule.expected_condition,
                verify_args,
                verification_tool_result.success,
                trim_for_log(&verification_tool_result.output)
            ),
            if verified { 1 } else { 0 }
        ],
    )
    .map_err(|e| format!("No se pudo insertar trace_verification: {}", e))?;

    Ok(())
}

fn insert_trace_request(
    conn: &Connection,
    trace_id: &str,
    tool_name: &str,
    now: &str,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO trace_request (
            id, trace_id, session_id, user_message, normalized_message, query_category,
            specialty_detected, requires_live_state, latency_ms_total, created_at
        ) VALUES (?1, ?2, NULL, ?3, ?3, 'action_request', 'post_tool', 0, 0, ?4)",
        params![
            format!("trq_{}", Uuid::new_v4().simple()),
            trace_id,
            format!("post-verify:{}", tool_name),
            now
        ],
    )
    .map_err(|e| format!("No se pudo insertar trace_request: {}", e))?;
    Ok(())
}

pub fn format_verification_message(tool_name: &str, outcome: &VerificationOutcome) -> String {
    if outcome.skipped {
        return format!(
            "[POST_VERIFY]\ntool={}\nstatus=skipped\nreason={}\ndetails={}",
            tool_name, outcome.reason_code, outcome.verification_result
        );
    }

    format!(
        "[POST_VERIFY]\ntool={}\nstatus={}\nreason={}\nverify_tool={}\ndetails={}",
        tool_name,
        if outcome.verified { "verified" } else { "failed" },
        outcome.reason_code,
        outcome
            .verify_tool_name
            .clone()
            .unwrap_or_else(|| "none".to_string()),
        outcome.verification_result
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitutes_service_name_placeholder() {
        let args = json!({ "name": "Spooler" });
        let resolved = resolve_verify_args(r#"{"name":"<service_name>"}"#, &args);
        assert_eq!(resolved["name"], "Spooler");
    }

    #[test]
    fn evaluates_service_running_condition() {
        let tool_result = ToolResult {
            tool_name: "restart_service".to_string(),
            success: true,
            output: "ok".to_string(),
            error: None,
        };
        let verify_result = ToolResult {
            tool_name: "get_service_status".to_string(),
            success: true,
            output: r#"{"Status":"Running"}"#.to_string(),
            error: None,
        };

        assert!(evaluate_expected_condition(
            "service_running",
            "restart_service",
            &json!({"name":"Spooler"}),
            &tool_result,
            &verify_result,
        ));
    }

    #[test]
    fn evaluates_process_not_found_condition() {
        let tool_result = ToolResult {
            tool_name: "kill_process".to_string(),
            success: true,
            output: "terminated".to_string(),
            error: None,
        };
        let verify_result = ToolResult {
            tool_name: "get_process_detail".to_string(),
            success: false,
            output: String::new(),
            error: Some("Proceso no encontrado".to_string()),
        };

        assert!(evaluate_expected_condition(
            "process_not_found",
            "kill_process",
            &json!({"pid":123}),
            &tool_result,
            &verify_result,
        ));
    }

    #[test]
    fn marks_service_recovery_as_failed_when_service_does_not_return() {
        let tool_result = ToolResult {
            tool_name: "restart_service".to_string(),
            success: true,
            output: "service restarted".to_string(),
            error: None,
        };
        let verify_result = ToolResult {
            tool_name: "get_service_status".to_string(),
            success: true,
            output: r#"{"Status":"Stopped"}"#.to_string(),
            error: None,
        };

        assert!(!evaluate_expected_condition(
            "service_running",
            "restart_service",
            &json!({"name":"Spooler"}),
            &tool_result,
            &verify_result,
        ));
    }
}
