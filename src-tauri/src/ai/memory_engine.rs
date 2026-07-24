use crate::ai::function_calling::ToolInfo;
use crate::ai::live_state_retriever::LiveStateContext;
use crate::rag::memory::{MemoryFactDraft, MemorySnapshotDraft};
use crate::rag::models::{DecisionEnvelope, QueryAnalysis, RagConfig};
use crate::rag::retrieval::{query_category_code, specialty_code};
use crate::rag::storage::sqlite::ensure_database_ready;
use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

pub fn create_session() -> Result<String, String> {
    let config = RagConfig::default();
    let conn = ensure_database_ready(&config)?;
    create_session_with_conn(&conn)
}

pub fn load_latest_memory_summary(session_id: &str) -> Result<Option<String>, String> {
    let config = RagConfig::default();
    let conn = ensure_database_ready(&config)?;
    load_latest_memory_summary_with_conn(&conn, session_id)
}

pub fn build_memory_snapshot(
    user_message: &str,
    assistant_message: &str,
    analysis: &QueryAnalysis,
    decision: &DecisionEnvelope,
    live_state: &LiveStateContext,
    tools_used: &[ToolInfo],
) -> MemorySnapshotDraft {
    let mut facts = Vec::new();
    let mut tags = Vec::new();
    let mut component_states = Vec::new();
    let mut open_questions = Vec::new();

    tags.push(format!("specialty:{}", specialty_code(&decision.specialty)));
    tags.push(format!("decision:{:?}", decision.decision_mode).to_lowercase());
    tags.push(format!("risk:{:?}", decision.risk_level).to_lowercase());
    tags.push(format!("confidence:{:?}", decision.confidence_level).to_lowercase());

    facts.push(MemoryFactDraft {
        fact_type: "user_goal".to_string(),
        fact_key: "latest_user_request".to_string(),
        fact_value: trim_text(user_message, 180),
        confidence: 0.95,
    });
    facts.push(MemoryFactDraft {
        fact_type: "assistant_response".to_string(),
        fact_key: "latest_assistant_response".to_string(),
        fact_value: trim_text(assistant_message, 220),
        confidence: 0.80,
    });

    for entity in &analysis.entities {
        facts.push(MemoryFactDraft {
            fact_type: "entity".to_string(),
            fact_key: entity.clone(),
            fact_value: entity.clone(),
            confidence: 0.90,
        });
    }

    for symptom in &analysis.symptoms {
        facts.push(MemoryFactDraft {
            fact_type: "symptom".to_string(),
            fact_key: symptom.clone(),
            fact_value: symptom.clone(),
            confidence: 0.88,
        });
    }

    for observation in &live_state.observations {
        facts.push(MemoryFactDraft {
            fact_type: "live_observation".to_string(),
            fact_key: observation.clone(),
            fact_value: observation.clone(),
            confidence: 0.92,
        });
    }

    for flag in &live_state.conflict_flags {
        facts.push(MemoryFactDraft {
            fact_type: "conflict".to_string(),
            fact_key: flag.clone(),
            fact_value: flag.clone(),
            confidence: 0.95,
        });
    }

    component_states.push((
        "clarification_required".to_string(),
        decision.requires_clarification.to_string(),
    ));
    component_states.push((
        "live_state_required".to_string(),
        decision.requires_live_state.to_string(),
    ));
    component_states.push((
        "human_required".to_string(),
        decision.requires_human.to_string(),
    ));

    if decision.requires_clarification {
        open_questions.push("Falta precision operacional para ejecutar o responder con seguridad.".to_string());
    }
    if decision.requires_human {
        open_questions.push("La accion requiere escalamiento o aprobacion humana.".to_string());
    }

    let tool_actions = tools_used.iter().map(|tool| tool.name.clone()).collect::<Vec<_>>();
    let resolved = !decision.requires_clarification && !decision.requires_human;

    MemorySnapshotDraft {
        summary: format!(
            "Consulta {:?} en {}. Decision {:?} con confianza {:.2}. Entidades: {}. Observaciones vivas: {}.",
            analysis.query_category,
            specialty_code(&decision.specialty),
            decision.decision_mode,
            decision.confidence_score,
            if analysis.entities.is_empty() {
                "ninguna".to_string()
            } else {
                analysis.entities.join(", ")
            },
            if live_state.observations.is_empty() {
                "sin observaciones".to_string()
            } else {
                live_state.observations.join(" | ")
            }
        ),
        facts,
        tags,
        component_states,
        open_questions,
        tool_actions,
        resolved,
    }
}

pub fn persist_session_memory(
    session_id: &str,
    user_message: &str,
    assistant_message: &str,
    analysis: &QueryAnalysis,
    decision: &DecisionEnvelope,
    live_state: &LiveStateContext,
    tools_used: &[ToolInfo],
) -> Result<(), String> {
    let config = RagConfig::default();
    let conn = ensure_database_ready(&config)?;
    persist_session_memory_with_conn(
        &conn,
        session_id,
        user_message,
        assistant_message,
        analysis,
        decision,
        live_state,
        tools_used,
    )
}

fn create_session_with_conn(conn: &Connection) -> Result<String, String> {
    let session_id = format!("sess_{}", Uuid::new_v4().simple());
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO conversation_session (
            id, created_at, updated_at, status, latest_specialty, latest_risk_level
        ) VALUES (?1, ?2, ?2, 'active', 'unknown', 'r0')",
        params![session_id, now],
    )
    .map_err(|e| format!("No se pudo crear conversation_session: {}", e))?;

    Ok(session_id)
}

fn load_latest_memory_summary_with_conn(
    conn: &Connection,
    session_id: &str,
) -> Result<Option<String>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT summary
             FROM memory_snapshot
             WHERE session_id = ?1
             ORDER BY created_at DESC
             LIMIT 1",
        )
        .map_err(|e| format!("No se pudo preparar lectura de memoria: {}", e))?;

    stmt.query_row(params![session_id], |row| row.get::<_, String>(0))
        .map(Some)
        .or_else(|err| {
            if matches!(err, rusqlite::Error::QueryReturnedNoRows) {
                Ok(None)
            } else {
                Err(format!("No se pudo leer memory_snapshot: {}", err))
            }
        })
}

fn persist_session_memory_with_conn(
    conn: &Connection,
    session_id: &str,
    user_message: &str,
    assistant_message: &str,
    analysis: &QueryAnalysis,
    decision: &DecisionEnvelope,
    live_state: &LiveStateContext,
    tools_used: &[ToolInfo],
) -> Result<(), String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("No se pudo iniciar transaccion de memoria: {}", e))?;
    let now = Utc::now().to_rfc3339();

    tx.execute(
        "UPDATE conversation_session
         SET updated_at = ?2, latest_specialty = ?3, latest_risk_level = ?4
         WHERE id = ?1",
        params![
            session_id,
            now,
            specialty_code(&decision.specialty),
            format!("{:?}", decision.risk_level).to_lowercase()
        ],
    )
    .map_err(|e| format!("No se pudo actualizar conversation_session: {}", e))?;

    insert_conversation_message(&tx, session_id, "user", user_message, &now)?;
    insert_conversation_message(&tx, session_id, "assistant", assistant_message, &now)?;

    let draft = build_memory_snapshot(
        user_message,
        assistant_message,
        analysis,
        decision,
        live_state,
        tools_used,
    );
    let snapshot_id = format!("mems_{}", Uuid::new_v4().simple());

    tx.execute(
        "INSERT INTO memory_snapshot (
            id, session_id, summary, latest_intent, latest_specialty, risk_level,
            confidence, decision_mode, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            snapshot_id,
            session_id,
            draft.summary,
            query_category_code(&analysis.query_category),
            specialty_code(&decision.specialty),
            format!("{:?}", decision.risk_level).to_lowercase(),
            decision.confidence_score,
            format!("{:?}", decision.decision_mode).to_lowercase(),
            now
        ],
    )
    .map_err(|e| format!("No se pudo insertar memory_snapshot: {}", e))?;

    for fact in draft.facts {
        tx.execute(
            "INSERT INTO memory_fact (
                id, snapshot_id, fact_type, fact_key, fact_value, confidence
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                format!("mf_{}", Uuid::new_v4().simple()),
                snapshot_id,
                fact.fact_type,
                fact.fact_key,
                fact.fact_value,
                fact.confidence
            ],
        )
        .map_err(|e| format!("No se pudo insertar memory_fact: {}", e))?;
    }

    for tag in draft.tags {
        tx.execute(
            "INSERT INTO memory_tag (id, snapshot_id, tag_value) VALUES (?1, ?2, ?3)",
            params![format!("mt_{}", Uuid::new_v4().simple()), snapshot_id, tag],
        )
        .map_err(|e| format!("No se pudo insertar memory_tag: {}", e))?;
    }

    for (component_name, state_value) in draft.component_states {
        tx.execute(
            "INSERT INTO memory_component_state (
                id, snapshot_id, component_name, state_value
            ) VALUES (?1, ?2, ?3, ?4)",
            params![
                format!("mcs_{}", Uuid::new_v4().simple()),
                snapshot_id,
                component_name,
                state_value
            ],
        )
        .map_err(|e| format!("No se pudo insertar memory_component_state: {}", e))?;
    }

    for tool_name in draft.tool_actions {
        tx.execute(
            "INSERT INTO memory_action_history (
                id, session_id, tool_name, action_mode, result_status, created_at
            ) VALUES (?1, ?2, ?3, 'tool_execute', 'observed', ?4)",
            params![
                format!("mah_{}", Uuid::new_v4().simple()),
                session_id,
                tool_name,
                now
            ],
        )
        .map_err(|e| format!("No se pudo insertar memory_action_history: {}", e))?;
    }

    for question in draft.open_questions {
        tx.execute(
            "INSERT INTO memory_open_hypothesis (
                id, session_id, hypothesis_label, confidence, status, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                format!("moh_{}", Uuid::new_v4().simple()),
                session_id,
                question,
                if draft.resolved { 0.30 } else { 0.75 },
                if draft.resolved { "resolved" } else { "open" },
                now
            ],
        )
        .map_err(|e| format!("No se pudo insertar memory_open_hypothesis: {}", e))?;
    }

    tx.commit()
        .map_err(|e| format!("No se pudo confirmar transaccion de memoria: {}", e))?;
    Ok(())
}

fn insert_conversation_message(
    conn: &Connection,
    session_id: &str,
    role: &str,
    content: &str,
    created_at: &str,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO conversation_message (
            id, session_id, role, content, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            format!("cmsg_{}", Uuid::new_v4().simple()),
            session_id,
            role,
            trim_text(content, 4000),
            created_at
        ],
    )
    .map_err(|e| format!("No se pudo insertar conversation_message: {}", e))?;
    Ok(())
}

fn trim_text(text: &str, max_len: usize) -> String {
    if text.len() > max_len {
        text[..max_len].to_string()
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag::models::{
        ConfidenceLevel, DecisionEnvelope, DecisionMode, DomainSpecialty, QueryCategory, RiskLevel,
    };

    fn memory_test_conn() -> Connection {
        let config = RagConfig {
            enabled: false,
            db_filename: format!("kernelia_rag_memory_{}.db", Uuid::new_v4().simple()),
            migrations_dir: "migrations".to_string(),
            seeds_dir: "seeds".to_string(),
        };
        ensure_database_ready(&config).expect("db ready")
    }

    #[test]
    fn builds_snapshot_with_entities_and_observations() {
        let analysis = QueryAnalysis {
            normalized_text: "reinicia spooler".to_string(),
            query_category: QueryCategory::ActionRequest,
            specialty: DomainSpecialty::Services,
            urgency: "normal".to_string(),
            symptoms: vec!["print_failure".to_string()],
            entities: vec!["spooler".to_string()],
            ambiguity_score: 0.2,
            requires_clarification: false,
        };
        let decision = DecisionEnvelope {
            query_category: QueryCategory::ActionRequest,
            specialty: DomainSpecialty::Services,
            confidence_level: ConfidenceLevel::High,
            confidence_score: 0.86,
            risk_level: RiskLevel::R2,
            decision_mode: DecisionMode::Execute,
            requires_clarification: false,
            requires_live_state: true,
            requires_snapshot: false,
            requires_human: false,
            allowed_tools: vec!["restart_service".to_string()],
            denied_tools: Vec::new(),
            reason_codes: vec!["COMMAND_EVIDENCE".to_string()],
        };
        let live_state = LiveStateContext {
            specialty: Some(DomainSpecialty::Services),
            summary: vec!["spooler_running".to_string()],
            observations: vec!["spooler_running".to_string()],
            conflict_flags: Vec::new(),
            snapshot_source: None,
            current_state: serde_json::json!({}),
            last_snapshot: None,
        };
        let draft = build_memory_snapshot(
            "reinicia spooler",
            "Voy a revisar el servicio.",
            &analysis,
            &decision,
            &live_state,
            &[ToolInfo {
                name: "restart_service".to_string(),
                arguments: "{\"name\":\"Spooler\"}".to_string(),
            }],
        );

        assert!(draft.facts.iter().any(|fact| fact.fact_key == "spooler"));
        assert!(draft
            .tool_actions
            .iter()
            .any(|tool| tool == "restart_service"));
    }

    #[test]
    fn persists_session_memory_rows() {
        let conn = memory_test_conn();
        let session_id = create_session_with_conn(&conn).expect("session created");
        let analysis = QueryAnalysis {
            normalized_text: "consulta dns".to_string(),
            query_category: QueryCategory::ActionRequest,
            specialty: DomainSpecialty::Network,
            urgency: "normal".to_string(),
            symptoms: vec!["dns_failure".to_string()],
            entities: vec!["dns".to_string()],
            ambiguity_score: 0.1,
            requires_clarification: false,
        };
        let decision = DecisionEnvelope {
            query_category: QueryCategory::ActionRequest,
            specialty: DomainSpecialty::Network,
            confidence_level: ConfidenceLevel::High,
            confidence_score: 0.9,
            risk_level: RiskLevel::R0,
            decision_mode: DecisionMode::Execute,
            requires_clarification: false,
            requires_live_state: true,
            requires_snapshot: false,
            requires_human: false,
            allowed_tools: vec!["dns_lookup".to_string()],
            denied_tools: Vec::new(),
            reason_codes: vec!["COMMAND_EVIDENCE".to_string()],
        };
        let live_state = LiveStateContext {
            specialty: Some(DomainSpecialty::Network),
            summary: vec!["network_failed_checks=0".to_string()],
            observations: vec!["local_ip=192.168.1.20".to_string()],
            conflict_flags: Vec::new(),
            snapshot_source: Some("manual".to_string()),
            current_state: serde_json::json!({}),
            last_snapshot: None,
        };

        persist_session_memory_with_conn(
            &conn,
            &session_id,
            "consulta dns",
            "La resolucion DNS parece disponible.",
            &analysis,
            &decision,
            &live_state,
            &[ToolInfo {
                name: "dns_lookup".to_string(),
                arguments: "{\"host\":\"example.com\"}".to_string(),
            }],
        )
        .expect("memory persisted");

        let conversation_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM conversation_message WHERE session_id = ?1",
                params![session_id],
                |row| row.get(0),
            )
            .expect("conversation count");
        let snapshot_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM memory_snapshot WHERE session_id = ?1",
                params![session_id],
                |row| row.get(0),
            )
            .expect("snapshot count");

        assert_eq!(conversation_count, 2);
        assert_eq!(snapshot_count, 1);
    }

    #[test]
    fn stores_conflict_when_memory_is_contradicted_by_live_state() {
        let analysis = QueryAnalysis {
            normalized_text: "spooler sigue fallando".to_string(),
            query_category: QueryCategory::ActionRequest,
            specialty: DomainSpecialty::Services,
            urgency: "normal".to_string(),
            symptoms: vec!["print_failure".to_string()],
            entities: vec!["spooler".to_string()],
            ambiguity_score: 0.15,
            requires_clarification: false,
        };
        let decision = DecisionEnvelope {
            query_category: QueryCategory::ActionRequest,
            specialty: DomainSpecialty::Services,
            confidence_level: ConfidenceLevel::Medium,
            confidence_score: 0.61,
            risk_level: RiskLevel::R2,
            decision_mode: DecisionMode::Simulate,
            requires_clarification: false,
            requires_live_state: true,
            requires_snapshot: false,
            requires_human: false,
            allowed_tools: vec!["restart_service".to_string()],
            denied_tools: Vec::new(),
            reason_codes: vec!["CONFLICT_DETECTED".to_string()],
        };
        let live_state = LiveStateContext {
            specialty: Some(DomainSpecialty::Services),
            summary: vec!["conflict=service_state_changed".to_string()],
            observations: vec!["spooler_running".to_string()],
            conflict_flags: vec!["service_state_changed".to_string()],
            snapshot_source: Some("manual".to_string()),
            current_state: serde_json::json!({}),
            last_snapshot: Some(serde_json::json!({
                "data": { "services": { "spooler": "stopped" } }
            })),
        };

        let draft = build_memory_snapshot(
            "el historial decia detenido pero ahora corre",
            "Detecto conflicto entre snapshot y estado vivo.",
            &analysis,
            &decision,
            &live_state,
            &[],
        );

        assert!(draft
            .facts
            .iter()
            .any(|fact| fact.fact_type == "conflict" && fact.fact_key == "service_state_changed"));
    }
}
