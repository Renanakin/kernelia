use crate::rag::models::{
    ConfidenceAssessment, ConfidenceLevel, DecisionEnvelope, DecisionMode, QueryAnalysis,
    QueryCategory, RiskLevel,
};
use crate::rag::retrieval::RetrievalBundle;

pub fn build_decision(
    analysis: &QueryAnalysis,
    retrieval: &RetrievalBundle,
    confidence: &ConfidenceAssessment,
) -> DecisionEnvelope {
    let risk_level = derive_risk_level(analysis, retrieval);
    let allowed_tools = derive_allowed_tools(retrieval, risk_level.clone());
    let denied_tools = derive_denied_tools(retrieval, risk_level.clone());

    let mut reason_codes = confidence.reason_codes.clone();
    reason_codes.push(format!("RISK_{:?}", risk_level).to_uppercase());

    let (decision_mode, requires_live_state, requires_snapshot, requires_human) =
        decide_mode(analysis, confidence, &risk_level, &allowed_tools);

    DecisionEnvelope {
        query_category: analysis.query_category.clone(),
        specialty: analysis.specialty.clone(),
        confidence_level: confidence.level.clone(),
        confidence_score: confidence.score,
        risk_level,
        decision_mode,
        requires_clarification: confidence.should_ask_clarifying_question,
        requires_live_state,
        requires_snapshot,
        requires_human,
        allowed_tools,
        denied_tools,
        reason_codes,
    }
}

fn decide_mode(
    analysis: &QueryAnalysis,
    confidence: &ConfidenceAssessment,
    risk_level: &RiskLevel,
    allowed_tools: &[String],
) -> (DecisionMode, bool, bool, bool) {
    if matches!(analysis.query_category, QueryCategory::OutOfDomain) {
        return (DecisionMode::Deny, false, false, false);
    }

    if confidence.should_ask_clarifying_question {
        return (DecisionMode::Clarify, false, false, false);
    }

    if matches!(analysis.query_category, QueryCategory::UnsafeRequest) {
        return match risk_level {
            RiskLevel::R4 => (DecisionMode::Deny, true, true, true),
            RiskLevel::R3 => (DecisionMode::Escalate, true, true, true),
            _ => (DecisionMode::Simulate, true, true, true),
        };
    }

    if matches!(analysis.query_category, QueryCategory::ActionRequest) {
        return match confidence.level {
            ConfidenceLevel::High => match risk_level {
                RiskLevel::R0 | RiskLevel::R1 => {
                    (DecisionMode::Execute, !allowed_tools.is_empty(), false, false)
                }
                RiskLevel::R2 => (DecisionMode::Simulate, true, false, false),
                RiskLevel::R3 | RiskLevel::R4 => (DecisionMode::Escalate, true, true, true),
            },
            ConfidenceLevel::Medium => match risk_level {
                RiskLevel::R0 | RiskLevel::R1 => (DecisionMode::Simulate, true, false, false),
                _ => (DecisionMode::Escalate, true, true, true),
            },
            ConfidenceLevel::Low => (DecisionMode::Clarify, false, false, false),
        };
    }

    match confidence.level {
        ConfidenceLevel::High => (DecisionMode::Explain, true, false, false),
        ConfidenceLevel::Medium => (DecisionMode::Explain, false, false, false),
        ConfidenceLevel::Low => (DecisionMode::Clarify, false, false, false),
    }
}

fn derive_risk_level(analysis: &QueryAnalysis, retrieval: &RetrievalBundle) -> RiskLevel {
    let mut max_risk = RiskLevel::R0;

    for hit in &retrieval.policy_hits {
        if hit.source_type != "risk_policy" {
            continue;
        }

        if let Some(entity_key) = &hit.entity_key {
            let risk = parse_risk_level(entity_key);
            if risk_rank(&risk) > risk_rank(&max_risk) {
                max_risk = risk;
            }
        }
    }

    if matches!(analysis.query_category, QueryCategory::UnsafeRequest) && risk_rank(&max_risk) < 4 {
        max_risk = RiskLevel::R4;
    }

    if analysis.specialty == crate::rag::models::DomainSpecialty::SensitiveOps
        && risk_rank(&max_risk) < 3
    {
        max_risk = RiskLevel::R3;
    }

    max_risk
}

fn derive_allowed_tools(retrieval: &RetrievalBundle, risk_level: RiskLevel) -> Vec<String> {
    retrieval
        .command_hits
        .iter()
        .filter_map(|hit| extract_tool_name(&hit.title))
        .filter(|tool_name| match risk_level {
            RiskLevel::R0 | RiskLevel::R1 => true,
            RiskLevel::R2 => !is_high_risk_tool(tool_name),
            RiskLevel::R3 | RiskLevel::R4 => false,
        })
        .collect()
}

fn derive_denied_tools(retrieval: &RetrievalBundle, risk_level: RiskLevel) -> Vec<String> {
    retrieval
        .command_hits
        .iter()
        .filter_map(|hit| extract_tool_name(&hit.title))
        .filter(|tool_name| match risk_level {
            RiskLevel::R0 | RiskLevel::R1 => false,
            RiskLevel::R2 => is_high_risk_tool(tool_name),
            RiskLevel::R3 | RiskLevel::R4 => true,
        })
        .collect()
}

fn extract_tool_name(title: &str) -> Option<String> {
    title
        .split(" -> ")
        .next()
        .map(str::trim)
        .filter(|value| value.starts_with("tool_") || value.contains('_'))
        .map(|value| value.to_string())
}

fn is_high_risk_tool(tool_name: &str) -> bool {
    [
        "disable_firewall",
        "run_shell_command",
        "run_powershell_command",
        "reboot_system",
        "shutdown_system",
        "kill_process",
        "disable_service",
        "write_file",
        "delete_file",
    ]
    .iter()
    .any(|candidate| tool_name.contains(candidate))
}

fn parse_risk_level(raw: &str) -> RiskLevel {
    match raw.to_lowercase().as_str() {
        "r1" => RiskLevel::R1,
        "r2" => RiskLevel::R2,
        "r3" => RiskLevel::R3,
        "r4" => RiskLevel::R4,
        _ => RiskLevel::R0,
    }
}

fn risk_rank(risk: &RiskLevel) -> i32 {
    match risk {
        RiskLevel::R0 => 0,
        RiskLevel::R1 => 1,
        RiskLevel::R2 => 2,
        RiskLevel::R3 => 3,
        RiskLevel::R4 => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag::models::{DomainSpecialty, RetrievalHit};

    fn hit(
        source_type: &str,
        title: &str,
        specialty: DomainSpecialty,
        entity_key: Option<&str>,
    ) -> RetrievalHit {
        RetrievalHit {
            source_type: source_type.to_string(),
            source_id: "id".to_string(),
            title: title.to_string(),
            score_lexical: 0.9,
            score_vector: 0.0,
            score_final: 0.9,
            specialty,
            entity_key: entity_key.map(|value| value.to_string()),
            content: "content".to_string(),
        }
    }

    #[test]
    fn chooses_clarify_for_low_confidence_queries() {
        let analysis = QueryAnalysis {
            normalized_text: "ayuda".to_string(),
            query_category: QueryCategory::Short,
            specialty: DomainSpecialty::Unknown,
            urgency: "normal".to_string(),
            symptoms: Vec::new(),
            entities: Vec::new(),
            ambiguity_score: 0.9,
            requires_clarification: true,
        };

        let confidence = ConfidenceAssessment {
            level: ConfidenceLevel::Low,
            score: 0.20,
            reason_codes: vec!["CLARIFICATION_REQUIRED".to_string()],
            should_use_context: false,
            should_ask_clarifying_question: true,
        };

        let decision = build_decision(&analysis, &RetrievalBundle::default(), &confidence);
        assert_eq!(decision.decision_mode, DecisionMode::Clarify);
    }

    #[test]
    fn chooses_execute_for_safe_high_confidence_action() {
        let analysis = QueryAnalysis {
            normalized_text: "consulta dns".to_string(),
            query_category: QueryCategory::ActionRequest,
            specialty: DomainSpecialty::Network,
            urgency: "normal".to_string(),
            symptoms: vec!["dns_failure".to_string()],
            entities: vec!["dns".to_string()],
            ambiguity_score: 0.2,
            requires_clarification: false,
        };

        let confidence = ConfidenceAssessment {
            level: ConfidenceLevel::High,
            score: 0.85,
            reason_codes: vec!["COMMAND_EVIDENCE".to_string()],
            should_use_context: true,
            should_ask_clarifying_question: false,
        };

        let retrieval = RetrievalBundle {
            knowledge_hits: Vec::new(),
            command_hits: vec![hit(
                "command_or_tool",
                "dns_lookup -> nslookup",
                DomainSpecialty::Network,
                None,
            )],
            policy_hits: vec![hit(
                "risk_policy",
                "Risk policy dns_lookup",
                DomainSpecialty::Network,
                Some("r0"),
            )],
            memory_hits: Vec::new(),
        };

        let decision = build_decision(&analysis, &retrieval, &confidence);
        assert_eq!(decision.decision_mode, DecisionMode::Execute);
        assert!(decision.allowed_tools.iter().any(|tool| tool.contains("dns_lookup")));
    }

    #[test]
    fn chooses_escalate_for_high_risk_action() {
        let analysis = QueryAnalysis {
            normalized_text: "deshabilita firewall".to_string(),
            query_category: QueryCategory::ActionRequest,
            specialty: DomainSpecialty::Security,
            urgency: "high".to_string(),
            symptoms: Vec::new(),
            entities: vec!["firewall".to_string()],
            ambiguity_score: 0.2,
            requires_clarification: false,
        };

        let confidence = ConfidenceAssessment {
            level: ConfidenceLevel::High,
            score: 0.88,
            reason_codes: vec!["POLICY_EVIDENCE".to_string()],
            should_use_context: true,
            should_ask_clarifying_question: false,
        };

        let retrieval = RetrievalBundle {
            knowledge_hits: Vec::new(),
            command_hits: vec![hit(
                "command_or_tool",
                "disable_firewall -> netsh",
                DomainSpecialty::Security,
                None,
            )],
            policy_hits: vec![hit(
                "risk_policy",
                "Risk policy disable_firewall",
                DomainSpecialty::Security,
                Some("r4"),
            )],
            memory_hits: Vec::new(),
        };

        let decision = build_decision(&analysis, &retrieval, &confidence);
        assert_eq!(decision.decision_mode, DecisionMode::Escalate);
        assert!(decision.requires_human);
    }

    #[test]
    fn denies_unsafe_r4_requests() {
        let analysis = QueryAnalysis {
            normalized_text: "borra protecciones y apaga seguridad".to_string(),
            query_category: QueryCategory::UnsafeRequest,
            specialty: DomainSpecialty::SensitiveOps,
            urgency: "high".to_string(),
            symptoms: Vec::new(),
            entities: vec!["firewall".to_string()],
            ambiguity_score: 0.1,
            requires_clarification: false,
        };

        let confidence = ConfidenceAssessment {
            level: ConfidenceLevel::High,
            score: 0.91,
            reason_codes: vec!["POLICY_EVIDENCE".to_string()],
            should_use_context: true,
            should_ask_clarifying_question: false,
        };

        let retrieval = RetrievalBundle {
            knowledge_hits: Vec::new(),
            command_hits: vec![hit(
                "command_or_tool",
                "disable_firewall -> netsh",
                DomainSpecialty::SensitiveOps,
                None,
            )],
            policy_hits: vec![hit(
                "risk_policy",
                "Risk policy disable_firewall",
                DomainSpecialty::SensitiveOps,
                Some("r4"),
            )],
            memory_hits: Vec::new(),
        };

        let decision = build_decision(&analysis, &retrieval, &confidence);
        assert_eq!(decision.decision_mode, DecisionMode::Deny);
        assert!(decision.requires_snapshot);
        assert!(decision.requires_human);
    }

    #[test]
    fn escalates_r3_action_without_snapshot() {
        let analysis = QueryAnalysis {
            normalized_text: "reinicia servicio critico".to_string(),
            query_category: QueryCategory::ActionRequest,
            specialty: DomainSpecialty::SensitiveOps,
            urgency: "high".to_string(),
            symptoms: vec!["service_failure".to_string()],
            entities: vec!["servicio".to_string()],
            ambiguity_score: 0.1,
            requires_clarification: false,
        };

        let confidence = ConfidenceAssessment {
            level: ConfidenceLevel::High,
            score: 0.84,
            reason_codes: vec!["COMMAND_EVIDENCE".to_string()],
            should_use_context: true,
            should_ask_clarifying_question: false,
        };

        let retrieval = RetrievalBundle {
            knowledge_hits: Vec::new(),
            command_hits: vec![hit(
                "command_or_tool",
                "restart_service -> sc",
                DomainSpecialty::SensitiveOps,
                None,
            )],
            policy_hits: vec![hit(
                "risk_policy",
                "Risk policy restart_service",
                DomainSpecialty::SensitiveOps,
                Some("r3"),
            )],
            memory_hits: Vec::new(),
        };

        let decision = build_decision(&analysis, &retrieval, &confidence);
        assert_eq!(decision.decision_mode, DecisionMode::Escalate);
        assert!(decision.requires_snapshot);
        assert!(decision.requires_human);
    }
}
