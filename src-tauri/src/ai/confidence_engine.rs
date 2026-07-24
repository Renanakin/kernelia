use crate::rag::models::{ConfidenceAssessment, ConfidenceLevel, QueryAnalysis, QueryCategory};
use crate::rag::retrieval::RetrievalBundle;

pub fn assess_confidence(
    analysis: &QueryAnalysis,
    retrieval: &RetrievalBundle,
) -> ConfidenceAssessment {
    let top_knowledge = retrieval
        .knowledge_hits
        .iter()
        .map(|hit| hit.score_final)
        .max_by(|a, b| a.total_cmp(b))
        .unwrap_or(0.0);
    let top_command = retrieval
        .command_hits
        .iter()
        .map(|hit| hit.score_final)
        .max_by(|a, b| a.total_cmp(b))
        .unwrap_or(0.0);
    let top_policy = retrieval
        .policy_hits
        .iter()
        .map(|hit| hit.score_final)
        .max_by(|a, b| a.total_cmp(b))
        .unwrap_or(0.0);

    let has_knowledge = !retrieval.knowledge_hits.is_empty();
    let has_commands = !retrieval.command_hits.is_empty();
    let has_policies = !retrieval.policy_hits.is_empty();

    let mut score = 0.0_f32;
    let mut reason_codes = Vec::new();

    if has_knowledge {
        score += top_knowledge * 0.40;
        reason_codes.push("KNOWLEDGE_EVIDENCE".to_string());
    } else {
        reason_codes.push("KNOWLEDGE_MISSING".to_string());
    }

    if has_commands {
        score += top_command * 0.30;
        reason_codes.push("COMMAND_EVIDENCE".to_string());
    }

    if has_policies {
        score += top_policy * 0.20;
        reason_codes.push("POLICY_EVIDENCE".to_string());
    } else {
        reason_codes.push("POLICY_MISSING".to_string());
    }

    if analysis.specialty != crate::rag::models::DomainSpecialty::Unknown {
        score += 0.08;
        reason_codes.push("SPECIALTY_RESOLVED".to_string());
    } else {
        score -= 0.10;
        reason_codes.push("SPECIALTY_UNKNOWN".to_string());
    }

    if !analysis.entities.is_empty() {
        score += 0.05;
        reason_codes.push("ENTITY_RESOLVED".to_string());
    }

    if analysis.urgency == "high" {
        score -= 0.05;
        reason_codes.push("URGENCY_REQUIRES_CAUTION".to_string());
    }

    if analysis.requires_clarification {
        score -= 0.22;
        reason_codes.push("CLARIFICATION_REQUIRED".to_string());
    }

    if matches!(
        analysis.query_category,
        QueryCategory::Short | QueryCategory::Ambiguous | QueryCategory::OutOfDomain
    ) {
        score -= 0.10;
        reason_codes.push("QUERY_LOW_PRECISION".to_string());
    }

    let score = score.clamp(0.0, 1.0);
    let level = if score >= 0.78 {
        ConfidenceLevel::High
    } else if score >= 0.55 {
        ConfidenceLevel::Medium
    } else {
        ConfidenceLevel::Low
    };

    let should_use_context = has_knowledge || has_commands || has_policies;
    let should_ask_clarifying_question = analysis.requires_clarification
        || matches!(
            analysis.query_category,
            QueryCategory::Short | QueryCategory::Ambiguous
        )
        || (!has_knowledge && !has_commands);

    ConfidenceAssessment {
        level,
        score,
        reason_codes,
        should_use_context,
        should_ask_clarifying_question,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag::models::{DomainSpecialty, RetrievalHit};

    fn hit(source_type: &str, score_final: f32, specialty: DomainSpecialty) -> RetrievalHit {
        RetrievalHit {
            source_type: source_type.to_string(),
            source_id: "id".to_string(),
            title: "title".to_string(),
            score_lexical: score_final,
            score_vector: 0.0,
            score_final,
            specialty,
            entity_key: None,
            content: "content".to_string(),
        }
    }

    #[test]
    fn raises_high_confidence_when_retrieval_is_consistent() {
        let analysis = QueryAnalysis {
            normalized_text: "no tengo internet revisa dns".to_string(),
            query_category: QueryCategory::ActionRequest,
            specialty: crate::rag::models::DomainSpecialty::Network,
            urgency: "normal".to_string(),
            symptoms: vec!["dns_failure".to_string()],
            entities: vec!["dns".to_string()],
            ambiguity_score: 0.20,
            requires_clarification: false,
        };

        let retrieval = RetrievalBundle {
            knowledge_hits: vec![hit("knowledge_chunk", 0.92, DomainSpecialty::Network)],
            command_hits: vec![hit("command_or_tool", 0.88, DomainSpecialty::Network)],
            policy_hits: vec![hit("decision_policy", 0.80, DomainSpecialty::Network)],
            memory_hits: Vec::new(),
        };

        let confidence = assess_confidence(&analysis, &retrieval);
        assert_eq!(confidence.level, ConfidenceLevel::High);
        assert!(!confidence.should_ask_clarifying_question);
    }

    #[test]
    fn lowers_confidence_for_ambiguous_queries() {
        let analysis = QueryAnalysis {
            normalized_text: "ayuda".to_string(),
            query_category: QueryCategory::Short,
            specialty: crate::rag::models::DomainSpecialty::Unknown,
            urgency: "normal".to_string(),
            symptoms: Vec::new(),
            entities: Vec::new(),
            ambiguity_score: 0.90,
            requires_clarification: true,
        };

        let confidence = assess_confidence(&analysis, &RetrievalBundle::default());
        assert_eq!(confidence.level, ConfidenceLevel::Low);
        assert!(confidence.should_ask_clarifying_question);
    }
}
