use crate::rag::ingest::{default_corpus_root, ingest_markdown_corpus_with_conn};
use crate::rag::models::{DomainSpecialty, QueryAnalysis, RagConfig, RetrievalHit};
use crate::rag::retrieval::{
    cosine_similarity, generate_embedding, hybrid_score, normalize_text, semantic_query_text,
    sort_and_trim, specialty_code, specialty_id, RetrievalBundle,
};
use crate::rag::storage::sqlite::ensure_database_ready;
use rusqlite::{params, Connection};
#[cfg(test)]
use uuid::Uuid;

pub fn retrieve_knowledge(analysis: &QueryAnalysis) -> RetrievalBundle {
    let config = RagConfig::default();
    let Ok(conn) = ensure_database_ready(&config) else {
        return RetrievalBundle::default();
    };

    let _ = ingest_markdown_corpus_with_conn(&conn, &default_corpus_root());
    retrieve_knowledge_with_conn(&conn, analysis).unwrap_or_default()
}

pub fn retrieve_knowledge_with_conn(
    conn: &Connection,
    analysis: &QueryAnalysis,
) -> Result<RetrievalBundle, String> {
    let mut knowledge_hits = query_knowledge_hits(conn, analysis)?;
    let mut policy_hits = query_policy_hits(conn, analysis)?;

    sort_and_trim(&mut knowledge_hits, 6);
    sort_and_trim(&mut policy_hits, 4);

    Ok(RetrievalBundle {
        knowledge_hits,
        policy_hits,
        ..Default::default()
    })
}

fn query_knowledge_hits(conn: &Connection, analysis: &QueryAnalysis) -> Result<Vec<RetrievalHit>, String> {
    let specialty_filter = specialty_id(&analysis.specialty);
    let like_query = format!("%{}%", analysis.normalized_text);
    let entity_filter = analysis.entities.first().cloned().unwrap_or_default();
    let query_embedding = generate_embedding(&semantic_query_text(analysis));
    let query_norm = normalize_text(&analysis.normalized_text);
    let query_tokens = significant_tokens(&query_norm);

    let mut stmt = conn
        .prepare(
            "SELECT
                kc.id,
                kd.title,
                kc.chunk_text,
                ds.code,
                COALESCE(kc.entity_key, ''),
                kc.title_anchor,
                kc.lexical_weight,
                kc.semantic_weight,
                COALESCE(kce.embedding_json, '[]')
             FROM knowledge_chunk kc
             INNER JOIN knowledge_document kd ON kd.id = kc.document_id
             INNER JOIN domain_specialty ds ON ds.id = kc.specialty_id
             LEFT JOIN knowledge_chunk_embedding kce ON kce.chunk_id = kc.id
             WHERE
                kc.chunk_text LIKE ?1
                OR kd.title LIKE ?1
                OR kc.title_anchor LIKE ?1
                OR (?2 <> '' AND COALESCE(kc.entity_key, '') = ?2)
                OR (?3 <> 'sp_unknown' AND kc.specialty_id = ?3)
                OR kce.embedding_json <> '[]'",
        )
        .map_err(|e| format!("No se pudo preparar retrieval de knowledge: {}", e))?;

    let rows = stmt
        .query_map(params![like_query, entity_filter, specialty_filter], |row| {
            let source_id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let chunk_text: String = row.get(2)?;
            let specialty: String = row.get(3)?;
            let entity_key: String = row.get(4)?;
            let title_anchor: String = row.get(5)?;
            let lexical_weight: f32 = row.get(6)?;
            let semantic_weight: f32 = row.get(7)?;
            let embedding_json: String = row.get(8)?;
            Ok((
                source_id,
                title,
                chunk_text,
                specialty,
                entity_key,
                title_anchor,
                lexical_weight,
                semantic_weight,
                embedding_json,
            ))
        })
        .map_err(|e| format!("No se pudo ejecutar retrieval de knowledge: {}", e))?;

    let mut hits = Vec::new();
    for row in rows {
        let (
            source_id,
            title,
            chunk_text,
            specialty,
            entity_key,
            title_anchor,
            lexical_weight,
            semantic_weight,
            embedding_json,
        ) =
            row.map_err(|e| format!("Fila invalida en knowledge retrieval: {}", e))?;
        let specialty = specialty_from_code(&specialty);
        let base_text = format!("{} {} {}", title, title_anchor, chunk_text);
        let base_norm = normalize_text(&base_text);
        let doc_tokens = significant_tokens(&base_norm);
        let lexical = crate::rag::retrieval::lexical_score(&base_text, analysis);
        let chunk_embedding: Vec<f32> =
            serde_json::from_str(&embedding_json).unwrap_or_else(|_| Vec::new());
        let semantic = if chunk_embedding.is_empty() {
            cosine_similarity(&query_embedding, &generate_embedding(&base_norm))
        } else {
            cosine_similarity(&query_embedding, &chunk_embedding)
        };
        let exact_query_bonus = exact_phrase_bonus(&query_norm, &base_norm);
        let token_overlap_bonus = token_overlap_bonus(&query_tokens, &doc_tokens);
        let anchor_bonus = anchor_bonus(&query_tokens, &title, &title_anchor);
        let entity_bonus = if !entity_filter.is_empty()
            && entity_key.eq_ignore_ascii_case(&entity_filter)
        {
            0.18
        } else {
            0.0
        };
        let score_final = (hybrid_score(
            &base_text,
            &specialty,
            analysis,
            semantic,
            lexical_weight,
            semantic_weight,
        ) + entity_bonus + exact_query_bonus + token_overlap_bonus + anchor_bonus)
            .clamp(0.0, 1.5);

        if score_final <= 0.20
            && lexical <= 0.0
            && semantic <= 0.15
            && exact_query_bonus <= 0.0
            && token_overlap_bonus <= 0.0
        {
            continue;
        }

        hits.push(RetrievalHit {
            source_type: "knowledge_chunk".to_string(),
            source_id,
            title,
            score_lexical: lexical,
            score_vector: semantic,
            score_final,
            specialty,
            entity_key: if entity_key.is_empty() { None } else { Some(entity_key) },
            content: chunk_text,
        });
    }

    Ok(hits)
}

fn query_policy_hits(conn: &Connection, analysis: &QueryAnalysis) -> Result<Vec<RetrievalHit>, String> {
    let specialty_code = specialty_code(&analysis.specialty);
    let query_category = crate::rag::retrieval::query_category_code(&analysis.query_category);

    let mut hits = Vec::new();

    let mut decision_stmt = conn
        .prepare(
            "SELECT dp.id, ds.code, qc.code, dp.decision_mode, dp.response_style
             FROM decision_policy dp
             INNER JOIN domain_specialty ds ON ds.id = dp.specialty_id
             INNER JOIN query_category qc ON qc.id = dp.query_category_id
             WHERE ds.code = ?1 OR qc.code = ?2",
        )
        .map_err(|e| format!("No se pudo preparar retrieval de decision policy: {}", e))?;

    let decision_rows = decision_stmt
        .query_map(params![specialty_code, query_category], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .map_err(|e| format!("No se pudo ejecutar retrieval de decision policy: {}", e))?;

    for row in decision_rows {
        let (id, specialty, category, mode, style) =
            row.map_err(|e| format!("Fila invalida de decision policy: {}", e))?;
        let candidate_specialty = specialty_from_code(&specialty);
        let content = format!(
            "decision_mode={} response_style={} specialty={} query_category={}",
            mode, style, specialty, category
        );
        let lexical = crate::rag::retrieval::lexical_score(&content, analysis);
        let score_final = (lexical
            + if specialty == specialty_code { 0.35 } else { 0.0 }
            + if category == query_category { 0.35 } else { 0.0 })
            .clamp(0.0, 1.2);

        hits.push(RetrievalHit {
            source_type: "decision_policy".to_string(),
            source_id: id,
            title: format!("Decision policy {} / {}", specialty, category),
            score_lexical: lexical,
            score_vector: 0.0,
            score_final,
            specialty: candidate_specialty,
            entity_key: None,
            content,
        });
    }

    let mut confidence_stmt = conn
        .prepare(
            "SELECT cp.id, ds.code, qc.code, cp.high_threshold, cp.medium_threshold
             FROM confidence_policy cp
             INNER JOIN domain_specialty ds ON ds.id = cp.specialty_id
             INNER JOIN query_category qc ON qc.id = cp.query_category_id
             WHERE ds.code = ?1 OR qc.code = ?2",
        )
        .map_err(|e| format!("No se pudo preparar retrieval de confidence policy: {}", e))?;

    let confidence_rows = confidence_stmt
        .query_map(params![specialty_code, query_category], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, f32>(3)?,
                row.get::<_, f32>(4)?,
            ))
        })
        .map_err(|e| format!("No se pudo ejecutar retrieval de confidence policy: {}", e))?;

    for row in confidence_rows {
        let (id, specialty, category, high_threshold, medium_threshold) =
            row.map_err(|e| format!("Fila invalida de confidence policy: {}", e))?;
        let candidate_specialty = specialty_from_code(&specialty);
        let content = format!(
            "high_threshold={} medium_threshold={} specialty={} query_category={}",
            high_threshold, medium_threshold, specialty, category
        );
        let score_final: f32 = (0.30_f32
            + if specialty == specialty_code { 0.30_f32 } else { 0.0_f32 }
            + if category == query_category { 0.30_f32 } else { 0.0_f32 })
            .clamp(0.0_f32, 1.0_f32);

        hits.push(RetrievalHit {
            source_type: "confidence_policy".to_string(),
            source_id: id,
            title: format!("Confidence policy {} / {}", specialty, category),
            score_lexical: 0.30,
            score_vector: 0.0,
            score_final,
            specialty: candidate_specialty,
            entity_key: None,
            content,
        });
    }

    Ok(hits)
}

fn exact_phrase_bonus(query_norm: &str, doc_norm: &str) -> f32 {
    if query_norm.is_empty() || doc_norm.is_empty() {
        return 0.0;
    }

    if doc_norm == query_norm || doc_norm.contains(query_norm) || query_norm.contains(doc_norm) {
        return 0.28;
    }

    0.0
}

fn token_overlap_bonus(query_tokens: &[String], doc_tokens: &[String]) -> f32 {
    if query_tokens.is_empty() || doc_tokens.is_empty() {
        return 0.0;
    }

    let shared = query_tokens
        .iter()
        .filter(|token| doc_tokens.iter().any(|candidate| candidate == *token))
        .count();
    let coverage = shared as f32 / query_tokens.len() as f32;

    if coverage >= 0.75 {
        0.18
    } else if coverage >= 0.50 {
        0.12
    } else if coverage >= 0.25 {
        0.06
    } else {
        0.0
    }
}

fn anchor_bonus(query_tokens: &[String], title: &str, title_anchor: &str) -> f32 {
    let title_norm = normalize_text(title);
    let anchor_norm = normalize_text(title_anchor);
    let title_tokens = significant_tokens(&title_norm);
    let anchor_tokens = significant_tokens(&anchor_norm);

    let title_shared = query_tokens
        .iter()
        .filter(|token| title_tokens.iter().any(|candidate| candidate == *token))
        .count();
    let anchor_shared = query_tokens
        .iter()
        .filter(|token| anchor_tokens.iter().any(|candidate| candidate == *token))
        .count();

    if title_shared >= 2 || anchor_shared >= 2 {
        0.10
    } else if title_shared >= 1 || anchor_shared >= 1 {
        0.05
    } else {
        0.0
    }
}

fn significant_tokens(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter(|token| token.len() >= 2 && !is_stopword(token))
        .map(|token| token.to_string())
        .collect()
}

fn is_stopword(token: &str) -> bool {
    matches!(
        token,
        "que"
            | "es"
            | "un"
            | "una"
            | "el"
            | "la"
            | "los"
            | "las"
            | "de"
            | "del"
            | "y"
            | "en"
            | "por"
            | "para"
            | "con"
            | "sobre"
            | "como"
            | "a"
            | "al"
            | "se"
            | "lo"
            | "me"
            | "mi"
            | "tu"
            | "su"
            | "te"
            | "hay"
            | "tener"
            | "tengo"
            | "tiene"
    )
}

fn specialty_from_code(code: &str) -> DomainSpecialty {
    match code {
        "system" => DomainSpecialty::System,
        "telemetry" => DomainSpecialty::Telemetry,
        "network" => DomainSpecialty::Network,
        "processes" => DomainSpecialty::Processes,
        "services" => DomainSpecialty::Services,
        "maintenance" => DomainSpecialty::Maintenance,
        "security" => DomainSpecialty::Security,
        "drivers" => DomainSpecialty::Drivers,
        "filesystem" => DomainSpecialty::Filesystem,
        "audit" => DomainSpecialty::Audit,
        "performance" => DomainSpecialty::Performance,
        "software" => DomainSpecialty::Software,
        "sensitive_ops" => DomainSpecialty::SensitiveOps,
        "megaboss" => DomainSpecialty::Megaboss,
        _ => DomainSpecialty::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::query_analyzer::analyze_query;
    use std::fs;
    use std::time::Instant;

    fn test_conn() -> Connection {
        let config = RagConfig {
            enabled: false,
            db_filename: format!("kernelia_rag_retrieval_{}.db", Uuid::new_v4().simple()),
            migrations_dir: "migrations".to_string(),
            seeds_dir: "seeds".to_string(),
        };
        let conn = ensure_database_ready(&config).expect("db ready");
        ingest_markdown_corpus_with_conn(&conn, &default_corpus_root()).expect("corpus ingested");
        conn
    }

    #[test]
    fn retrieves_network_knowledge_by_dns() {
        let conn = test_conn();
        let analysis = analyze_query("no tengo internet revisa dns");
        let bundle = retrieve_knowledge_with_conn(&conn, &analysis).expect("knowledge bundle");

        assert!(!bundle.knowledge_hits.is_empty());
        assert_eq!(bundle.knowledge_hits[0].specialty, DomainSpecialty::Network);
    }

    #[test]
    fn retrieves_service_policy_for_spooler_context() {
        let conn = test_conn();
        let analysis = analyze_query("reinicia el spooler");
        let bundle = retrieve_knowledge_with_conn(&conn, &analysis).expect("knowledge bundle");

        assert!(bundle
            .policy_hits
            .iter()
            .any(|hit| hit.source_type == "decision_policy" || hit.source_type == "confidence_policy"));
    }

    #[test]
    fn retrieves_dns_knowledge_from_open_resolution_language() {
        let conn = test_conn();
        let analysis = analyze_query("tengo conectividad ip pero falla resolver nombres de paginas");
        let bundle = retrieve_knowledge_with_conn(&conn, &analysis).expect("knowledge bundle");

        assert!(!bundle.knowledge_hits.is_empty());
        assert!(bundle.knowledge_hits[0].title.to_lowercase().contains("dns"));
        assert!(bundle.knowledge_hits[0].score_vector > 0.20);
    }

    #[test]
    fn prefers_exact_dns_chunk_over_conflicting_generic_chunk() {
        let conn = test_conn();
        let temp_root = std::env::temp_dir().join(format!("kernelia_rag_conflict_{}", Uuid::new_v4().simple()));
        fs::create_dir_all(temp_root.join("network")).expect("temp corpus dir");

        let exact_doc = r#"---
title: DNS Hard Failure Exact
slug: dns-hard-failure-exact
specialty: network
doc_type: playbook
entity_key: dns
source_kind: curated_markdown
status: active
---
# Diagnostico
Usar este documento cuando falla DNS, resolucion de nombres y dominios.
"#;

        let generic_doc = r#"---
title: Network Generic Recovery
slug: network-generic-recovery
specialty: network
doc_type: guide
source_kind: curated_markdown
status: active
---
# Diagnostico
Usar esta guia para conectividad general, red lenta y ajustes basicos del adaptador.
"#;

        fs::write(temp_root.join("network").join("dns-exact.md"), exact_doc).expect("exact doc");
        fs::write(temp_root.join("network").join("network-generic.md"), generic_doc).expect("generic doc");
        ingest_markdown_corpus_with_conn(&conn, &temp_root).expect("temp corpus ingested");

        let analysis = analyze_query("falla dns y no resuelve dominios");
        let bundle = retrieve_knowledge_with_conn(&conn, &analysis).expect("knowledge bundle");

        assert!(!bundle.knowledge_hits.is_empty());
        assert_eq!(bundle.knowledge_hits[0].title, "DNS Hard Failure Exact");

        let _ = fs::remove_dir_all(temp_root);
    }

    #[test]
    fn retrieval_latency_stays_bounded_for_curated_corpus() {
        let conn = test_conn();
        let analysis = analyze_query("internet intermitente y error al resolver nombres");
        let started = Instant::now();

        let bundle = retrieve_knowledge_with_conn(&conn, &analysis).expect("knowledge bundle");
        let elapsed = started.elapsed().as_millis();

        assert!(!bundle.knowledge_hits.is_empty());
        assert!(elapsed < 500, "retrieval took {} ms", elapsed);
    }
}
