use crate::ai::live_state_retriever::LiveStateContext;
use crate::ai::specialty_agent::{render_specialty_agent_context, SpecialtyAgentProfile};
use crate::rag::models::{DecisionEnvelope, QueryAnalysis, RetrievalHit};
use crate::rag::retrieval::RetrievalBundle;

pub fn build_prompt_context(
    analysis: &QueryAnalysis,
    decision: &DecisionEnvelope,
    retrieval: &RetrievalBundle,
    live_state: &LiveStateContext,
    specialty_agent: &SpecialtyAgentProfile,
    memory_summary: Option<&str>,
) -> String {
    let knowledge = render_hits("KNOWLEDGE", &retrieval.knowledge_hits, 3);
    let commands = render_hits("COMMANDS", &retrieval.command_hits, 3);
    let policies = render_hits("POLICIES", &retrieval.policy_hits, 3);

    let live_state_summary = if live_state.summary.is_empty() {
        "sin estado vivo adicional".to_string()
    } else {
        live_state.summary.join(" | ")
    };

    let observations = if live_state.observations.is_empty() {
        "sin observaciones".to_string()
    } else {
        live_state.observations.join(" | ")
    };

    let conflicts = if live_state.conflict_flags.is_empty() {
        "sin conflictos detectados".to_string()
    } else {
        live_state.conflict_flags.join(" | ")
    };

    let allowed_tools = if decision.allowed_tools.is_empty() {
        "ninguna".to_string()
    } else {
        decision.allowed_tools.join(", ")
    };

    let denied_tools = if decision.denied_tools.is_empty() {
        "ninguna".to_string()
    } else {
        decision.denied_tools.join(", ")
    };

    let memory_summary = memory_summary.unwrap_or("sin memoria operacional previa");
    let recommendation = render_recommendation(decision);

    format!(
        "[KERNEL_RAG_CONTEXT]\nquery_category={:?}\nspecialty={:?}\nurgency={}\nentities={}\nsymptoms={}\nambiguity_score={:.2}\n\n[RAG_DIRECTIVES]\n1. Responde a la pregunta utilizando el contexto local [KNOWLEDGE], [COMMANDS] y [POLICIES].\n2. Si el contexto local no contiene la respuesta completa, consulta las fuentes oficiales de Microsoft (site:learn.microsoft.com OR site:support.microsoft.com).\n3. La respuesta DEBE ser siempre ordenada y estructurada en dos secciones principales:\n   - ### Solución\n   - ### Consejos y Recomendaciones\n4. PROHIBIDO REPETIR LAS INSTRUCCIONES DEL PROMPT, PLANTILLAS INTERNAS O SALUDOS GENÉRICOS DE BIENVENIDA. Responde DIRECTAMENTE con la Solución y Consejos.\n\n[KERNEL_MEMORY]\n{}\n\n[KERNEL_DECISION]\nmode={:?}\nconfidence_level={:?}\nconfidence_score={:.2}\nrisk_level={:?}\nrequires_clarification={}\nrequires_live_state={}\nrequires_snapshot={}\nrequires_human={}\nallowed_tools={}\ndenied_tools={}\nreason_codes={}\n\n[KERNEL_RECOMMENDATION]\n{}\n\n[KERNEL_LIVE_STATE]\nsource={}\nsummary={}\nobservations={}\nconflicts={}\n\n{}\n\n{}\n\n{}\n\n{}",
        analysis.query_category,
        decision.specialty,
        analysis.urgency,
        if analysis.entities.is_empty() {
            "none".to_string()
        } else {
            analysis.entities.join(", ")
        },
        if analysis.symptoms.is_empty() {
            "none".to_string()
        } else {
            analysis.symptoms.join(", ")
        },
        analysis.ambiguity_score,
        memory_summary,
        decision.decision_mode,
        decision.confidence_level,
        decision.confidence_score,
        decision.risk_level,
        decision.requires_clarification,
        decision.requires_live_state,
        decision.requires_snapshot,
        decision.requires_human,
        allowed_tools,
        denied_tools,
        if decision.reason_codes.is_empty() {
            "none".to_string()
        } else {
            decision.reason_codes.join(", ")
        },
        recommendation,
        live_state
            .snapshot_source
            .clone()
            .unwrap_or_else(|| "live_only".to_string()),
        live_state_summary,
        observations,
        conflicts,
        render_specialty_agent_context(specialty_agent, decision, retrieval, live_state),
        knowledge,
        commands,
        policies,
    )
}

fn render_recommendation(decision: &DecisionEnvelope) -> String {
    match decision.decision_mode {
        crate::rag::models::DecisionMode::Clarify => {
            "Hacer una sola pregunta de aclaracion concreta antes de continuar.".to_string()
        }
        crate::rag::models::DecisionMode::Explain => {
            "Responder con evidencia local, diagnostico breve y una recomendacion accionable.".to_string()
        }
        crate::rag::models::DecisionMode::Simulate => {
            "Explicar el plan y mostrar el resultado esperado sin ejecutar cambios.".to_string()
        }
        crate::rag::models::DecisionMode::Execute => {
            "Usar solo tools permitidas, resumir la evidencia y cerrar con la accion realizada.".to_string()
        }
        crate::rag::models::DecisionMode::Escalate => {
            "No improvisar. Escalar al especialista correcto con la evidencia recolectada.".to_string()
        }
        crate::rag::models::DecisionMode::Deny => {
            "Bloquear la accion y explicar el motivo tecnico o de seguridad.".to_string()
        }
    }
}

fn render_hits(section_name: &str, hits: &[RetrievalHit], limit: usize) -> String {
    let lines = hits
        .iter()
        .take(limit)
        .enumerate()
        .map(|(idx, hit)| {
            format!(
                "Document {}:::\n- [{}] {} | score={:.2} | {}",
                idx + 1,
                hit.source_type,
                hit.title,
                hit.score_final,
                compact_content(&hit.content)
            )
        })
        .collect::<Vec<_>>();

    if lines.is_empty() {
        format!("[{}]\n- none", section_name)
    } else {
        format!("[{}]\n{}", section_name, lines.join("\n\n"))
    }
}

fn compact_content(content: &str) -> String {
    let trimmed = content.split_whitespace().collect::<Vec<_>>().join(" ");
    if trimmed.len() > 220 {
        format!("{}...", &trimmed[..220])
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::specialty_agent::build_specialty_agent_profile;
    use crate::rag::models::{
        ConfidenceLevel, DecisionMode, DomainSpecialty, QueryCategory, RetrievalHit, RiskLevel,
    };

    #[test]
    fn includes_governed_sections_in_prompt_context() {
        let analysis = QueryAnalysis {
            normalized_text: "consulta dns".to_string(),
            query_category: QueryCategory::ActionRequest,
            specialty: DomainSpecialty::Network,
            urgency: "normal".to_string(),
            symptoms: vec!["dns_failure".to_string()],
            entities: vec!["dns".to_string()],
            ambiguity_score: 0.20,
            requires_clarification: false,
        };

        let decision = DecisionEnvelope {
            query_category: QueryCategory::ActionRequest,
            specialty: DomainSpecialty::Network,
            confidence_level: ConfidenceLevel::High,
            confidence_score: 0.88,
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

        let retrieval = RetrievalBundle {
            knowledge_hits: vec![RetrievalHit {
                source_type: "knowledge_chunk".to_string(),
                source_id: "k1".to_string(),
                title: "DNS recovery".to_string(),
                score_lexical: 0.8,
                score_vector: 0.0,
                score_final: 0.9,
                specialty: DomainSpecialty::Network,
                entity_key: Some("dns".to_string()),
                content: "Validar gateway y ejecutar consulta DNS.".to_string(),
            }],
            command_hits: vec![RetrievalHit {
                source_type: "command_or_tool".to_string(),
                source_id: "c1".to_string(),
                title: "dns_lookup -> nslookup".to_string(),
                score_lexical: 0.8,
                score_vector: 0.0,
                score_final: 0.92,
                specialty: DomainSpecialty::Network,
                entity_key: Some("consulta dns".to_string()),
                content: "nslookup {host}".to_string(),
            }],
            policy_hits: vec![RetrievalHit {
                source_type: "risk_policy".to_string(),
                source_id: "p1".to_string(),
                title: "Risk policy dns_lookup".to_string(),
                score_lexical: 0.7,
                score_vector: 0.0,
                score_final: 0.85,
                specialty: DomainSpecialty::Network,
                entity_key: Some("r0".to_string()),
                content: "viewer r0 allow_auto_execute".to_string(),
            }],
            memory_hits: Vec::new(),
        };

        let live_state = LiveStateContext {
            specialty: Some(DomainSpecialty::Network),
            summary: vec!["network_failed_checks=0".to_string()],
            observations: vec!["local_ip=192.168.1.10".to_string()],
            conflict_flags: Vec::new(),
            snapshot_source: Some("manual".to_string()),
            current_state: serde_json::json!({}),
            last_snapshot: None,
        };
        let specialist = build_specialty_agent_profile(&DomainSpecialty::Network);

        let prompt = build_prompt_context(
            &analysis,
            &decision,
            &retrieval,
            &live_state,
            &specialist,
            Some("memoria previa"),
        );
        assert!(prompt.contains("[KERNEL_DECISION]"));
        assert!(prompt.contains("[KERNEL_RECOMMENDATION]"));
        assert!(prompt.contains("[KERNEL_MEMORY]"));
        assert!(prompt.contains("allowed_tools=dns_lookup"));
        assert!(prompt.contains("[COMMANDS]"));
        assert!(prompt.contains("[POLICIES]"));
        assert!(prompt.contains("[KERNEL_SPECIALIST]"));
    }
}
