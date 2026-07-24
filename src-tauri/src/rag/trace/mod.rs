use crate::rag::models::{ConfidenceAssessment, DecisionEnvelope, QueryAnalysis, RetrievalHit};

#[derive(Debug, Clone, Default)]
pub struct TraceDraft {
    pub trace_id: String,
    pub session_id: Option<String>,
    pub user_message: String,
    pub normalized_query: String,
    pub query_category: String,
    pub specialty_detected: String,
    pub requires_live_state: bool,
    pub latency_ms_total: i64,
    pub retrieval_hits: Vec<TraceRetrievalHit>,
    pub confidence: Option<TraceConfidenceDraft>,
    pub decision: Option<TraceDecisionDraft>,
    pub errors: Vec<TraceErrorDraft>,
}

#[derive(Debug, Clone)]
pub struct TraceRetrievalHit {
    pub source_type: String,
    pub source_id: String,
    pub title: String,
    pub score_vector: f32,
    pub score_lexical: f32,
    pub score_final: f32,
    pub entity_key: Option<String>,
    pub used_in_context: bool,
}

#[derive(Debug, Clone)]
pub struct TraceConfidenceDraft {
    pub confidence_level: String,
    pub confidence_score: f32,
    pub should_use_context: bool,
    pub should_ask_clarifying_question: bool,
}

#[derive(Debug, Clone)]
pub struct TraceDecisionDraft {
    pub decision_mode: String,
    pub reason_text: String,
    pub used_context: bool,
    pub used_live_state: bool,
    pub used_tools: bool,
    pub escalated: bool,
}

#[derive(Debug, Clone)]
pub struct TraceErrorDraft {
    pub stage_name: String,
    pub error_code: String,
    pub error_message: String,
}

impl TraceRetrievalHit {
    pub fn from_hit(hit: &RetrievalHit, used_in_context: bool) -> Self {
        Self {
            source_type: hit.source_type.clone(),
            source_id: hit.source_id.clone(),
            title: hit.title.clone(),
            score_vector: hit.score_vector,
            score_lexical: hit.score_lexical,
            score_final: hit.score_final,
            entity_key: hit.entity_key.clone(),
            used_in_context,
        }
    }
}

impl TraceConfidenceDraft {
    pub fn from_confidence(confidence: &ConfidenceAssessment) -> Self {
        Self {
            confidence_level: format!("{:?}", confidence.level).to_lowercase(),
            confidence_score: confidence.score,
            should_use_context: confidence.should_use_context,
            should_ask_clarifying_question: confidence.should_ask_clarifying_question,
        }
    }
}

impl TraceDecisionDraft {
    pub fn from_decision(
        analysis: &QueryAnalysis,
        decision: &DecisionEnvelope,
        confidence: &ConfidenceAssessment,
    ) -> Self {
        let mut reasons = decision.reason_codes.clone();
        for reason in &confidence.reason_codes {
            if !reasons.iter().any(|existing| existing == reason) {
                reasons.push(reason.clone());
            }
        }

        Self {
            decision_mode: format!("{:?}", decision.decision_mode).to_lowercase(),
            reason_text: reasons.join(", "),
            used_context: confidence.should_use_context,
            used_live_state: decision.requires_live_state,
            used_tools: !decision.allowed_tools.is_empty()
                && matches!(analysis.query_category, crate::rag::models::QueryCategory::ActionRequest),
            escalated: decision.requires_human
                || matches!(
                    decision.decision_mode,
                    crate::rag::models::DecisionMode::Escalate | crate::rag::models::DecisionMode::Deny
                ),
        }
    }
}
