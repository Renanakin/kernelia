use crate::rag::models::RagConfig;
use crate::rag::retrieval::generate_embedding;
use crate::rag::storage::sqlite::ensure_database_ready;
use rusqlite::params;
use uuid::Uuid;

pub fn ingest_user_validated_solution(
    query_text: &str,
    solution_text: &str,
    specialty_code: &str,
) -> Result<String, String> {
    let config = RagConfig::default();
    let conn = ensure_database_ready(&config)?;

    let now = chrono::Utc::now().to_rfc3339();
    let doc_id = format!("doc-user-val-{}", Uuid::new_v4());
    let chunk_id = format!("chunk-user-val-{}", Uuid::new_v4());
    let embed_id = format!("embed-user-val-{}", Uuid::new_v4());
    let title = format!("Solución Validada: {}", query_text);
    let slug = format!("solucion-validada-{}", Uuid::new_v4());
    let chunk_text = format!("Problema: {}\nSolución Validada: {}", query_text, solution_text);

    // 1. Insertar Documento
    conn.execute(
        "INSERT INTO knowledge_document (id, specialty_id, doc_type, title, slug, summary, body_markdown, source_kind, source_path, content_hash, created_at, updated_at)
         VALUES (?1, ?2, 'user_validated', ?3, ?4, ?3, ?5, 'user_feedback', 'auto_ingest', 'hash', ?6, ?6)",
        params![doc_id, specialty_code, title, slug, solution_text, now],
    ).map_err(|e| format!("No se pudo crear documento de solucion validada: {}", e))?;

    // 2. Insertar Chunk
    conn.execute(
        "INSERT INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, title_anchor, lexical_weight, semantic_weight)
         VALUES (?1, ?2, 0, ?3, ?4, ?5, 1.5, 1.5)",
        params![chunk_id, doc_id, chunk_text, specialty_code, title],
    ).map_err(|e| format!("No se pudo crear chunk de solucion validada: {}", e))?;

    // 3. Generar y guardar embeddings
    let vec_embed = generate_embedding(&chunk_text);
    let embed_json = serde_json::to_string(&vec_embed).unwrap_or_else(|_| "[]".to_string());

    conn.execute(
        "INSERT INTO knowledge_chunk_embedding (id, chunk_id, embedding_provider, embedding_model, embedding_json, created_at)
         VALUES (?1, ?2, 'all-minilm-l6-v2', 'AllMiniLmL6V2', ?3, ?4)",
        params![embed_id, chunk_id, embed_json, now],
    ).map_err(|e| format!("No se pudo crear embedding de solucion validada: {}", e))?;

    // 4. Registrar log de feedback
    let feedback_id = format!("fb-{}", Uuid::new_v4());
    conn.execute(
        "INSERT INTO user_feedback_log (id, query_text, solution_text, satisfied, source_type, created_at)
         VALUES (?1, ?2, ?3, 1, 'web_validated', ?4)",
        params![feedback_id, query_text, solution_text, now],
    ).map_err(|e| format!("No se pudo registrar log de feedback: {}", e))?;

    Ok(chunk_id)
}
