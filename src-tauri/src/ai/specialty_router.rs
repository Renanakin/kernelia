use crate::rag::models::{DomainSpecialty, QueryAnalysis, QueryCategory};

pub fn route_specialty(analysis: &QueryAnalysis) -> DomainSpecialty {
    if analysis.specialty != DomainSpecialty::Unknown {
        return analysis.specialty.clone();
    }

    if analysis.entities.iter().any(|entity| entity == "spooler") {
        return DomainSpecialty::Services;
    }

    if analysis.entities.iter().any(|entity| entity == "dns") {
        return DomainSpecialty::Network;
    }

    if analysis.entities.iter().any(|entity| entity == "gpu") {
        return DomainSpecialty::Drivers;
    }

    if analysis.symptoms.iter().any(|symptom| symptom == "disk_100" || symptom == "high_cpu") {
        return DomainSpecialty::Performance;
    }

    if matches!(analysis.query_category, QueryCategory::UnsafeRequest) {
        return DomainSpecialty::SensitiveOps;
    }

    DomainSpecialty::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_services_from_spooler_entity() {
        let analysis = QueryAnalysis {
            normalized_text: "no imprime".to_string(),
            query_category: QueryCategory::SymptomBased,
            specialty: DomainSpecialty::Unknown,
            urgency: "normal".to_string(),
            symptoms: vec!["print_failure".to_string()],
            entities: vec!["spooler".to_string()],
            ambiguity_score: 0.45,
            requires_clarification: false,
        };

        assert_eq!(route_specialty(&analysis), DomainSpecialty::Services);
    }

    #[test]
    fn infers_sensitive_ops_for_unsafe_requests() {
        let analysis = QueryAnalysis {
            normalized_text: "borra system32".to_string(),
            query_category: QueryCategory::UnsafeRequest,
            specialty: DomainSpecialty::Unknown,
            urgency: "high".to_string(),
            symptoms: Vec::new(),
            entities: Vec::new(),
            ambiguity_score: 0.20,
            requires_clarification: false,
        };

        assert_eq!(route_specialty(&analysis), DomainSpecialty::SensitiveOps);
    }
}
