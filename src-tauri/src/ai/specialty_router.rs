use crate::rag::models::{DomainSpecialty, QueryAnalysis, QueryCategory};

pub fn route_specialty(analysis: &QueryAnalysis) -> DomainSpecialty {
    let list = route_specialties_multi(analysis);
    list.first().cloned().unwrap_or(DomainSpecialty::Unknown)
}

/// Enrutamiento Multi-Especialista autónomo para escenarios complejos
pub fn route_specialties_multi(analysis: &QueryAnalysis) -> Vec<DomainSpecialty> {
    let mut list = Vec::new();

    if analysis.specialty != DomainSpecialty::Unknown {
        list.push(analysis.specialty.clone());
    }

    if analysis.entities.iter().any(|e| e == "spooler" || e == "printer" || e == "servicio") {
        if !list.contains(&DomainSpecialty::Services) {
            list.push(DomainSpecialty::Services);
        }
    }

    if analysis.entities.iter().any(|e| e == "dns" || e == "ip" || e == "gateway" || e == "wifi")
        || analysis.symptoms.iter().any(|s| s.contains("net") || s.contains("dns"))
    {
        if !list.contains(&DomainSpecialty::Network) {
            list.push(DomainSpecialty::Network);
        }
    }

    if analysis.entities.iter().any(|e| e == "gpu" || e == "driver" || e == "pnp")
        || analysis.symptoms.iter().any(|s| s.contains("driver") || s.contains("code_43"))
    {
        if !list.contains(&DomainSpecialty::Drivers) {
            list.push(DomainSpecialty::Drivers);
        }
    }

    if analysis.symptoms.iter().any(|s| s == "disk_100" || s == "high_cpu" || s == "lag") {
        if !list.contains(&DomainSpecialty::Performance) {
            list.push(DomainSpecialty::Performance);
        }
        if !list.contains(&DomainSpecialty::Processes) {
            list.push(DomainSpecialty::Processes);
        }
    }

    if matches!(analysis.query_category, QueryCategory::UnsafeRequest) {
        if !list.contains(&DomainSpecialty::SensitiveOps) {
            list.push(DomainSpecialty::SensitiveOps);
        }
    }

    if list.is_empty() {
        vec![DomainSpecialty::Unknown]
    } else {
        list
    }
}

/// Genera el informe de colaboración multi-especialista para el prompt y la auditoría
pub fn format_multi_specialist_plan(specialties: &[DomainSpecialty]) -> String {
    if specialties.len() <= 1 {
        format!("Especialista asignado: {:?}", specialties.first().unwrap_or(&DomainSpecialty::Unknown))
    } else {
        let names = specialties
            .iter()
            .map(|s| format!("{:?}", s))
            .collect::<Vec<_>>()
            .join(" -> ");
        format!("Cadena de Orquestación Multi-Especialista: [{}]", names)
    }
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

    #[test]
    fn routes_multiple_specialties_for_complex_performance_issue() {
        let analysis = QueryAnalysis {
            normalized_text: "pc lenta disco 100 spooler atascado".to_string(),
            query_category: QueryCategory::SymptomBased,
            specialty: DomainSpecialty::Unknown,
            urgency: "high".to_string(),
            symptoms: vec!["disk_100".to_string()],
            entities: vec!["spooler".to_string()],
            ambiguity_score: 0.30,
            requires_clarification: false,
        };

        let mult = route_specialties_multi(&analysis);
        assert!(mult.contains(&DomainSpecialty::Services));
        assert!(mult.contains(&DomainSpecialty::Performance));
        assert!(mult.contains(&DomainSpecialty::Processes));

        let plan = format_multi_specialist_plan(&mult);
        assert!(plan.contains("Cadena de Orquestación Multi-Especialista"));
    }
}
