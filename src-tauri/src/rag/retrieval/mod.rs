use crate::rag::models::{DomainSpecialty, QueryAnalysis, QueryCategory, RetrievalHit};
use std::collections::HashMap;

const EMBEDDING_DIM: usize = 64;
const SEMANTIC_SYNONYMS: &[(&str, &[&str])] = &[
    ("dns", &["resolucion", "resolver", "dominio", "dominios", "nombre", "nombres"]),
    ("internet", &["navegacion", "conectividad", "conexion", "online"]),
    ("gateway", &["puerta", "enlace", "router"]),
    ("wifi", &["inalambrica", "wireless", "ssid"]),
    ("spooler", &["impresion", "imprimir", "impresora", "cola"]),
    ("driver", &["controlador", "dispositivo"]),
    ("gpu", &["video", "grafica", "pantalla"]),
    ("disco", &["almacenamiento", "io", "i/o"]),
    ("lento", &["lentitud", "bloqueo", "demora"]),
];

#[derive(Debug, Default, Clone)]
pub struct RetrievalBundle {
    pub knowledge_hits: Vec<RetrievalHit>,
    pub command_hits: Vec<RetrievalHit>,
    pub policy_hits: Vec<RetrievalHit>,
    pub memory_hits: Vec<RetrievalHit>,
}

pub fn specialty_code(specialty: &DomainSpecialty) -> &'static str {
    match specialty {
        DomainSpecialty::System => "system",
        DomainSpecialty::Telemetry => "telemetry",
        DomainSpecialty::Network => "network",
        DomainSpecialty::Processes => "processes",
        DomainSpecialty::Services => "services",
        DomainSpecialty::Maintenance => "maintenance",
        DomainSpecialty::Security => "security",
        DomainSpecialty::Drivers => "drivers",
        DomainSpecialty::Filesystem => "filesystem",
        DomainSpecialty::Audit => "audit",
        DomainSpecialty::Performance => "performance",
        DomainSpecialty::Software => "software",
        DomainSpecialty::SensitiveOps => "sensitive_ops",
        DomainSpecialty::Megaboss => "megaboss",
        DomainSpecialty::Unknown => "unknown",
    }
}

pub fn specialty_id(specialty: &DomainSpecialty) -> String {
    format!("sp_{}", specialty_code(specialty))
}

pub fn embedding_dim() -> usize {
    EMBEDDING_DIM
}

pub fn query_category_code(category: &QueryCategory) -> &'static str {
    match category {
        QueryCategory::Specific => "specific",
        QueryCategory::Short => "short",
        QueryCategory::Ambiguous => "ambiguous",
        QueryCategory::SymptomBased => "symptom_based",
        QueryCategory::ActionRequest => "action_request",
        QueryCategory::UnsafeRequest => "unsafe_request",
        QueryCategory::OutOfDomain => "out_of_domain",
    }
}

pub fn lexical_score(text: &str, analysis: &QueryAnalysis) -> f32 {
    let haystack = text.to_lowercase();
    let mut score = 0.0_f32;

    for token in analysis.normalized_text.split_whitespace() {
        if token.len() < 2 {
            continue;
        }
        if haystack.contains(token) {
            score += 0.18;
        }
    }

    for symptom in &analysis.symptoms {
        let symptom_text = symptom.replace('_', " ");
        if haystack.contains(&symptom_text) || haystack.contains(symptom) {
            score += 0.25;
        }
    }

    for entity in &analysis.entities {
        if haystack.contains(entity) {
            score += 0.30;
        }
    }

    score.min(1.0)
}

pub fn normalize_text(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|ch| match ch {
            'a'..='z' | '0'..='9' | ' ' => ch,
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            _ => ' ',
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn semantic_query_text(analysis: &QueryAnalysis) -> String {
    let mut parts = vec![analysis.normalized_text.clone()];
    parts.extend(analysis.symptoms.clone());
    parts.extend(analysis.entities.clone());

    let normalized = normalize_text(&parts.join(" "));
    let mut expanded = Vec::new();
    for token in normalized.split_whitespace() {
        expanded.push(token.to_string());
        for (anchor, synonyms) in SEMANTIC_SYNONYMS {
            if token == *anchor || synonyms.iter().any(|syn| syn == &token) {
                expanded.push((*anchor).to_string());
                expanded.extend(synonyms.iter().map(|syn| (*syn).to_string()));
            }
        }
    }

    expanded.join(" ")
}

pub fn generate_embedding(text: &str) -> Vec<f32> {
    let normalized = normalize_text(text);
    let mut vector = vec![0.0_f32; EMBEDDING_DIM];
    if normalized.is_empty() {
        return vector;
    }

    let mut counts: HashMap<String, f32> = HashMap::new();
    let words: Vec<&str> = normalized.split_whitespace().collect();

    for word in &words {
        *counts.entry((*word).to_string()).or_insert(0.0) += 1.0;
    }

    for pair in words.windows(2) {
        let bigram = format!("{}_{}", pair[0], pair[1]);
        *counts.entry(bigram).or_insert(0.0) += 0.8;
    }

    for (anchor, synonyms) in SEMANTIC_SYNONYMS {
        if normalized.contains(anchor) || synonyms.iter().any(|syn| normalized.contains(syn)) {
            *counts.entry((*anchor).to_string()).or_insert(0.0) += 1.5;
            for synonym in *synonyms {
                *counts.entry((*synonym).to_string()).or_insert(0.0) += 0.6;
            }
        }
    }

    for (token, weight) in counts {
        let index = stable_index(&token);
        vector[index] += weight;
    }

    normalize_vector(&mut vector);
    vector
}

pub fn cosine_similarity(left: &[f32], right: &[f32]) -> f32 {
    if left.len() != right.len() || left.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0_f32;
    let mut left_norm = 0.0_f32;
    let mut right_norm = 0.0_f32;

    for (a, b) in left.iter().zip(right.iter()) {
        dot += a * b;
        left_norm += a * a;
        right_norm += b * b;
    }

    if left_norm <= f32::EPSILON || right_norm <= f32::EPSILON {
        return 0.0;
    }

    (dot / (left_norm.sqrt() * right_norm.sqrt())).clamp(0.0, 1.0)
}

pub fn hybrid_score(
    base_text: &str,
    candidate_specialty: &DomainSpecialty,
    analysis: &QueryAnalysis,
    semantic_score: f32,
    lexical_weight: f32,
    semantic_weight: f32,
) -> f32 {
    let lexical = lexical_score(base_text, analysis) * lexical_weight.max(0.1);
    let exact = exact_match_bonus(base_text, analysis);
    let specialty = specialty_bonus(candidate_specialty, analysis);
    let semantic = semantic_score * semantic_weight.max(0.1);

    (lexical * 0.55 + semantic * 0.35 + exact + specialty - ambiguity_penalty(analysis)).clamp(0.0, 1.35)
}

fn stable_index(token: &str) -> usize {
    let mut hash: u64 = 1469598103934665603;
    for byte in token.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    (hash as usize) % EMBEDDING_DIM
}

fn normalize_vector(vector: &mut [f32]) {
    let norm = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
    if norm <= f32::EPSILON {
        return;
    }

    for value in vector.iter_mut() {
        *value /= norm;
    }
}

pub fn exact_match_bonus(text: &str, analysis: &QueryAnalysis) -> f32 {
    let haystack = text.to_lowercase();
    if haystack == analysis.normalized_text {
        return 0.35;
    }

    if analysis
        .entities
        .iter()
        .any(|entity| haystack.contains(entity) || entity.contains(&haystack))
    {
        return 0.20;
    }

    if analysis
        .symptoms
        .iter()
        .any(|symptom| haystack.contains(symptom) || haystack.contains(&symptom.replace('_', " ")))
    {
        return 0.15;
    }

    0.0
}

pub fn specialty_bonus(candidate_specialty: &DomainSpecialty, analysis: &QueryAnalysis) -> f32 {
    if analysis.specialty == DomainSpecialty::Unknown {
        return 0.0;
    }

    if candidate_specialty == &analysis.specialty {
        0.25
    } else {
        0.0
    }
}

pub fn ambiguity_penalty(analysis: &QueryAnalysis) -> f32 {
    analysis.ambiguity_score * 0.20
}

pub fn final_score(base_text: &str, candidate_specialty: &DomainSpecialty, analysis: &QueryAnalysis) -> f32 {
    let lexical = lexical_score(base_text, analysis);
    let exact = exact_match_bonus(base_text, analysis);
    let specialty = specialty_bonus(candidate_specialty, analysis);
    (lexical + exact + specialty - ambiguity_penalty(analysis)).clamp(0.0, 1.25)
}

pub fn sort_and_trim(hits: &mut Vec<RetrievalHit>, limit: usize) {
    hits.sort_by(|a, b| b.score_final.total_cmp(&a.score_final));
    hits.truncate(limit);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag::models::{DomainSpecialty, QueryCategory};

    #[test]
    fn semantic_embedding_links_dns_resolution_language() {
        let left = generate_embedding("dns resolucion de dominios");
        let right = generate_embedding("falla al resolver nombres de internet");
        assert!(cosine_similarity(&left, &right) > 0.40);
    }

    #[test]
    fn hybrid_score_rewards_semantic_matches() {
        let analysis = QueryAnalysis {
            normalized_text: "falla al resolver nombres".to_string(),
            query_category: QueryCategory::SymptomBased,
            specialty: DomainSpecialty::Network,
            urgency: "normal".to_string(),
            symptoms: vec!["dns_failure".to_string()],
            entities: Vec::new(),
            ambiguity_score: 0.1,
            requires_clarification: false,
        };

        let score = hybrid_score(
            "Conectividad IP correcta pero falla DNS y resolucion de dominios",
            &DomainSpecialty::Network,
            &analysis,
            0.72,
            1.0,
            1.0,
        );
        assert!(score > 0.65);
    }
}
