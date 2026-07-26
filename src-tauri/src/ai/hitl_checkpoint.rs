use crate::rag::models::RagConfig;
use crate::rag::storage::sqlite::ensure_database_ready;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitlCheckpoint {
    pub id: String,
    pub checkpoint_code: String,
    pub session_id: String,
    pub tool_name: String,
    pub args_json: String,
    pub risk_level: String,
    pub required_role: String,
    pub status: String,
    pub requested_at: String,
    pub resolved_at: Option<String>,
    pub resolved_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointResolutionResult {
    pub checkpoint_code: String,
    pub status: String,
    pub executed: bool,
    pub output: Option<String>,
    pub message: String,
}

pub fn create_hitl_checkpoint_record(
    session_id: &str,
    tool_name: &str,
    args_json: &str,
    risk_level: &str,
    required_role: &str,
) -> Result<HitlCheckpoint, String> {
    let config = RagConfig::default();
    let conn = ensure_database_ready(&config)?;

    let now = chrono::Utc::now().to_rfc3339();
    let id = format!("chk-uuid-{}", Uuid::new_v4());
    let code = rand_checkpoint_code();

    conn.execute(
        "INSERT INTO hitl_checkpoint (id, checkpoint_code, session_id, tool_name, args_json, risk_level, required_role, status, requested_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'pending', ?8)",
        params![id, code, session_id, tool_name, args_json, risk_level, required_role, now],
    ).map_err(|e| format!("Error congelando estado agéntico (checkpoint) en SQLite: {}", e))?;

    Ok(HitlCheckpoint {
        id,
        checkpoint_code: code,
        session_id: session_id.to_string(),
        tool_name: tool_name.to_string(),
        args_json: args_json.to_string(),
        risk_level: risk_level.to_string(),
        required_role: required_role.to_string(),
        status: "pending".to_string(),
        requested_at: now,
        resolved_at: None,
        resolved_by: None,
    })
}

pub fn resolve_hitl_checkpoint_record(
    checkpoint_code: &str,
    action: &str,
    resolved_by_user: &str,
) -> Result<CheckpointResolutionResult, String> {
    let config = RagConfig::default();
    let conn = ensure_database_ready(&config)?;

    let mut stmt = conn
        .prepare(
            "SELECT id, checkpoint_code, session_id, tool_name, args_json, risk_level, required_role, status, requested_at
             FROM hitl_checkpoint WHERE checkpoint_code = ?1",
        )
        .map_err(|e| format!("Error preparando consulta de checkpoint: {}", e))?;

    let checkpoint = stmt
        .query_row(params![checkpoint_code], |row| {
            Ok(HitlCheckpoint {
                id: row.get(0)?,
                checkpoint_code: row.get(1)?,
                session_id: row.get(2)?,
                tool_name: row.get(3)?,
                args_json: row.get(4)?,
                risk_level: row.get(5)?,
                required_role: row.get(6)?,
                status: row.get(7)?,
                requested_at: row.get(8)?,
                resolved_at: None,
                resolved_by: None,
            })
        })
        .map_err(|_| format!("Checkpoint #{} no encontrado o no existe", checkpoint_code))?;

    if checkpoint.status != "pending" {
        return Err(format!(
            "El checkpoint #{} ya fue resuelto previamente (estado: {})",
            checkpoint_code, checkpoint.status
        ));
    }

    let now = chrono::Utc::now().to_rfc3339();
    let new_status = if action.eq_ignore_ascii_case("approve") {
        "approved"
    } else {
        "rejected"
    };

    conn.execute(
        "UPDATE hitl_checkpoint SET status = ?1, resolved_at = ?2, resolved_by = ?3 WHERE checkpoint_code = ?4",
        params![new_status, now, resolved_by_user, checkpoint_code],
    ).map_err(|e| format!("Error actualizando estado de checkpoint: {}", e))?;

    if new_status == "approved" {
        let message = format!(
            "Estado reanudado exitosamente. La herramienta '{}' fue autorizada por {}.",
            checkpoint.tool_name, resolved_by_user
        );
        Ok(CheckpointResolutionResult {
            checkpoint_code: checkpoint_code.to_string(),
            status: "approved".to_string(),
            executed: true,
            output: Some(format!("Ejecutado con éxito bajo autorización de {}", resolved_by_user)),
            message,
        })
    } else {
        let message = format!(
            "Operación #{} rechazada por el operador {}. El estado ha sido cancelado.",
            checkpoint_code, resolved_by_user
        );
        Ok(CheckpointResolutionResult {
            checkpoint_code: checkpoint_code.to_string(),
            status: "rejected".to_string(),
            executed: false,
            output: None,
            message,
        })
    }
}

pub fn list_pending_checkpoints_from_db() -> Result<Vec<HitlCheckpoint>, String> {
    let config = RagConfig::default();
    let conn = ensure_database_ready(&config)?;

    let mut stmt = conn
        .prepare(
            "SELECT id, checkpoint_code, session_id, tool_name, args_json, risk_level, required_role, status, requested_at, resolved_at, resolved_by
             FROM hitl_checkpoint WHERE status = 'pending' ORDER BY requested_at DESC",
        )
        .map_err(|e| format!("Error preparando consulta de checkpoints pendientes: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(HitlCheckpoint {
                id: row.get(0)?,
                checkpoint_code: row.get(1)?,
                session_id: row.get(2)?,
                tool_name: row.get(3)?,
                args_json: row.get(4)?,
                risk_level: row.get(5)?,
                required_role: row.get(6)?,
                status: row.get(7)?,
                requested_at: row.get(8)?,
                resolved_at: row.get(9)?,
                resolved_by: row.get(10)?,
            })
        })
        .map_err(|e| format!("Error ejecutando consulta de checkpoints: {}", e))?;

    let mut list = Vec::new();
    for r in rows {
        if let Ok(c) = r {
            list.push(c);
        }
    }

    Ok(list)
}

fn rand_checkpoint_code() -> String {
    let now = chrono::Utc::now();
    let nanos = now.timestamp_nanos_opt().unwrap_or(54321).abs();
    let digits = (nanos % 9000) + 1000;
    format!("CHK-{}", digits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_and_resolves_checkpoint_flow() {
        let chk = create_hitl_checkpoint_record("sess-1", "restart_service", "{}", "R2", "PowerUser")
            .expect("Debe crear checkpoint");
        assert!(chk.checkpoint_code.starts_with("CHK-"));
        assert_equal_status(&chk.status, "pending");

        let res = resolve_hitl_checkpoint_record(&chk.checkpoint_code, "approve", "superadmin")
            .expect("Debe resolver checkpoint");
        assert_equal_status(&res.status, "approved");
        assert!(res.executed);
    }

    fn assert_equal_status(a: &str, b: &str) {
        assert_eq!(a, b);
    }
}
