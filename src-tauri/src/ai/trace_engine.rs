use crate::ai::function_calling::ToolInfo;
use crate::rag::models::{ConfidenceAssessment, DecisionEnvelope, QueryAnalysis, RagConfig};
use crate::rag::retrieval::{query_category_code, specialty_code, RetrievalBundle};
use crate::rag::storage::sqlite::ensure_database_ready;
use crate::rag::trace::{
    TraceConfidenceDraft, TraceDecisionDraft, TraceDraft, TraceErrorDraft, TraceRetrievalHit,
};
use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

pub fn build_trace(
    trace_id: &str,
    session_id: Option<&str>,
    user_message: &str,
    analysis: &QueryAnalysis,
    retrieval: &RetrievalBundle,
    confidence: &ConfidenceAssessment,
    decision: &DecisionEnvelope,
    latency_ms_total: i64,
) -> TraceDraft {
    let mut retrieval_hits = Vec::new();
    retrieval_hits.extend(
        retrieval
            .knowledge_hits
            .iter()
            .map(|hit| TraceRetrievalHit::from_hit(hit, true)),
    );
    retrieval_hits.extend(
        retrieval
            .command_hits
            .iter()
            .map(|hit| TraceRetrievalHit::from_hit(hit, true)),
    );
    retrieval_hits.extend(
        retrieval
            .policy_hits
            .iter()
            .map(|hit| TraceRetrievalHit::from_hit(hit, true)),
    );
    retrieval_hits.extend(
        retrieval
            .memory_hits
            .iter()
            .map(|hit| TraceRetrievalHit::from_hit(hit, false)),
    );

    TraceDraft {
        trace_id: trace_id.to_string(),
        session_id: session_id.map(str::to_string),
        user_message: user_message.to_string(),
        normalized_query: analysis.normalized_text.clone(),
        query_category: query_category_code(&analysis.query_category).to_string(),
        specialty_detected: specialty_code(&analysis.specialty).to_string(),
        requires_live_state: decision.requires_live_state,
        latency_ms_total,
        retrieval_hits,
        confidence: Some(TraceConfidenceDraft::from_confidence(confidence)),
        decision: Some(TraceDecisionDraft::from_decision(
            analysis, decision, confidence,
        )),
        errors: Vec::new(),
    }
}

pub fn append_error(
    trace: &mut TraceDraft,
    stage_name: &str,
    error_code: &str,
    error_message: &str,
) {
    trace.errors.push(TraceErrorDraft {
        stage_name: stage_name.to_string(),
        error_code: error_code.to_string(),
        error_message: error_message.to_string(),
    });
}

pub fn persist_trace(trace: &TraceDraft, tools_used: &[ToolInfo]) -> Result<(), String> {
    let conn = ensure_database_ready(&RagConfig::default())?;
    persist_trace_with_conn(&conn, trace, tools_used)
}

fn persist_trace_with_conn(
    conn: &Connection,
    trace: &TraceDraft,
    tools_used: &[ToolInfo],
) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO trace_request (
            id, trace_id, session_id, user_message, normalized_message, query_category,
            specialty_detected, requires_live_state, latency_ms_total, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            format!("trq_{}", Uuid::new_v4().simple()),
            trace.trace_id,
            trace.session_id,
            trace.user_message,
            trace.normalized_query,
            trace.query_category,
            trace.specialty_detected,
            bool_to_i64(trace.requires_live_state),
            trace.latency_ms_total,
            now
        ],
    )
    .map_err(|e| format!("No se pudo insertar trace_request: {}", e))?;

    for hit in &trace.retrieval_hits {
        conn.execute(
            "INSERT INTO trace_retrieval_hit (
                id, trace_id, source_type, source_id, title, score_vector, score_lexical,
                score_final, entity_key, used_in_context
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                format!("trh_{}", Uuid::new_v4().simple()),
                trace.trace_id,
                hit.source_type,
                hit.source_id,
                hit.title,
                hit.score_vector,
                hit.score_lexical,
                hit.score_final,
                hit.entity_key,
                bool_to_i64(hit.used_in_context)
            ],
        )
        .map_err(|e| format!("No se pudo insertar trace_retrieval_hit: {}", e))?;
    }

    if let Some(confidence) = &trace.confidence {
        conn.execute(
            "INSERT INTO trace_confidence (
                id, trace_id, confidence_level, confidence_score, should_use_context,
                should_ask_clarifying_question
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                format!("tcf_{}", Uuid::new_v4().simple()),
                trace.trace_id,
                confidence.confidence_level,
                confidence.confidence_score,
                bool_to_i64(confidence.should_use_context),
                bool_to_i64(confidence.should_ask_clarifying_question)
            ],
        )
        .map_err(|e| format!("No se pudo insertar trace_confidence: {}", e))?;
    }

    if let Some(decision) = &trace.decision {
        conn.execute(
            "INSERT INTO trace_decision (
                id, trace_id, decision_mode, reason_text, used_context, used_live_state,
                used_tools, escalated
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                format!("tdc_{}", Uuid::new_v4().simple()),
                trace.trace_id,
                decision.decision_mode,
                decision.reason_text,
                bool_to_i64(decision.used_context),
                bool_to_i64(decision.used_live_state),
                bool_to_i64(decision.used_tools),
                bool_to_i64(decision.escalated)
            ],
        )
        .map_err(|e| format!("No se pudo insertar trace_decision: {}", e))?;
    }

    for tool in tools_used {
        conn.execute(
            "INSERT INTO trace_tool_call (
                id, trace_id, tool_name, args_json, result_status, verification_status
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                format!("ttc_{}", Uuid::new_v4().simple()),
                trace.trace_id,
                tool.name,
                tool.arguments,
                "completed",
                "pending"
            ],
        )
        .map_err(|e| format!("No se pudo insertar trace_tool_call: {}", e))?;
    }

    for error in &trace.errors {
        conn.execute(
            "INSERT INTO trace_error (
                id, trace_id, stage_name, error_code, error_message, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                format!("ter_{}", Uuid::new_v4().simple()),
                trace.trace_id,
                error.stage_name,
                error.error_code,
                error.error_message,
                now
            ],
        )
        .map_err(|e| format!("No se pudo insertar trace_error: {}", e))?;
    }

    Ok(())
}

fn bool_to_i64(value: bool) -> i64 {
    if value { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag::models::{
        ConfidenceLevel, DecisionMode, DomainSpecialty, QueryCategory, RetrievalHit, RiskLevel,
    };

    fn sample_analysis() -> QueryAnalysis {
        QueryAnalysis {
            normalized_text: "sin internet revisar dns".to_string(),
            query_category: QueryCategory::ActionRequest,
            specialty: DomainSpecialty::Network,
            urgency: "normal".to_string(),
            symptoms: vec!["dns_failure".to_string()],
            entities: vec!["dns".to_string()],
            ambiguity_score: 0.1,
            requires_clarification: false,
        }
    }

    fn sample_hit(source_type: &str, title: &str) -> RetrievalHit {
        RetrievalHit {
            source_type: source_type.to_string(),
            source_id: format!("{}_id", source_type),
            title: title.to_string(),
            score_lexical: 0.82,
            score_vector: 0.0,
            score_final: 0.91,
            specialty: DomainSpecialty::Network,
            entity_key: Some("dns".to_string()),
            content: "contenido".to_string(),
        }
    }

    fn sample_retrieval() -> RetrievalBundle {
        RetrievalBundle {
            knowledge_hits: vec![sample_hit("knowledge_chunk", "DNS diagnosis")],
            command_hits: vec![sample_hit("command_or_tool", "dns_lookup -> nslookup")],
            policy_hits: vec![sample_hit("decision_policy", "Network policy")],
            memory_hits: Vec::new(),
        }
    }

    fn sample_confidence() -> ConfidenceAssessment {
        ConfidenceAssessment {
            level: ConfidenceLevel::High,
            score: 0.88,
            reason_codes: vec!["KNOWLEDGE_EVIDENCE".to_string()],
            should_use_context: true,
            should_ask_clarifying_question: false,
        }
    }

    fn sample_decision() -> DecisionEnvelope {
        DecisionEnvelope {
            query_category: QueryCategory::ActionRequest,
            specialty: DomainSpecialty::Network,
            confidence_level: ConfidenceLevel::High,
            confidence_score: 0.88,
            risk_level: RiskLevel::R1,
            decision_mode: DecisionMode::Execute,
            requires_clarification: false,
            requires_live_state: true,
            requires_snapshot: false,
            requires_human: false,
            allowed_tools: vec!["dns_lookup".to_string()],
            denied_tools: Vec::new(),
            reason_codes: vec!["COMMAND_EVIDENCE".to_string()],
        }
    }

    #[test]
    fn builds_trace_with_retrieval_confidence_and_decision() {
        let trace = build_trace(
            "trace_test_1",
            Some("session_1"),
            "No tengo internet",
            &sample_analysis(),
            &sample_retrieval(),
            &sample_confidence(),
            &sample_decision(),
            145,
        );

        assert_eq!(trace.query_category, "action_request");
        assert_eq!(trace.specialty_detected, "network");
        assert_eq!(trace.retrieval_hits.len(), 3);
        assert!(trace.confidence.is_some());
        assert!(trace.decision.is_some());
    }

    #[test]
    fn persists_trace_rows_into_sqlite() {
        let trace_id = format!("trace_test_{}", Uuid::new_v4().simple());
        let mut trace = build_trace(
            &trace_id,
            Some("session_trace"),
            "No tengo internet",
            &sample_analysis(),
            &sample_retrieval(),
            &sample_confidence(),
            &sample_decision(),
            210,
        );
        append_error(
            &mut trace,
            "response_generation",
            "LLM_TIMEOUT",
            "Tiempo de respuesta agotado",
        );

        let tools = vec![ToolInfo {
            name: "dns_lookup".to_string(),
            arguments: r#"{"host":"openai.com"}"#.to_string(),
        }];

        persist_trace(&trace, &tools).expect("trace should persist");

        let conn = ensure_database_ready(&RagConfig::default()).expect("sqlite should open");

        let request_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM trace_request WHERE trace_id = ?1",
                params![trace_id.clone()],
                |row| row.get(0),
            )
            .expect("request count");
        assert_eq!(request_count, 1);

        let retrieval_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM trace_retrieval_hit WHERE trace_id = ?1",
                params![trace_id.clone()],
                |row| row.get(0),
            )
            .expect("retrieval count");
        assert_eq!(retrieval_count, 3);

        let tool_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM trace_tool_call WHERE trace_id = ?1",
                params![trace_id.clone()],
                |row| row.get(0),
            )
            .expect("tool count");
        assert_eq!(tool_count, 1);

        let error_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM trace_error WHERE trace_id = ?1",
                params![trace_id],
                |row| row.get(0),
            )
            .expect("error count");
        assert_eq!(error_count, 1);
    }
}
