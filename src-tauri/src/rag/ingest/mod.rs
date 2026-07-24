use crate::rag::models::RagConfig;
use crate::rag::retrieval::{generate_embedding, normalize_text};
use crate::rag::storage::sqlite::ensure_database_ready;
use chrono::Utc;
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const MAX_CHUNK_LEN: usize = 900;

#[derive(Debug, Clone)]
pub struct IngestSource {
    pub source_kind: String,
    pub source_path: String,
    pub specialty: String,
}

#[derive(Debug, Clone)]
pub struct DocumentFrontmatter {
    pub title: String,
    pub slug: String,
    pub specialty: String,
    pub doc_type: String,
    pub entity_key: Option<String>,
    pub source_kind: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct IngestedDocument {
    pub frontmatter: DocumentFrontmatter,
    pub source_path: String,
    pub body_markdown: String,
    pub summary: String,
    pub content_hash: String,
    pub chunks: Vec<IngestedChunk>,
}

#[derive(Debug, Clone)]
pub struct IngestedChunk {
    pub chunk_index: usize,
    pub chunk_text: String,
    pub title_anchor: String,
    pub entity_key: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct IngestReport {
    pub corpus_root: String,
    pub files_discovered: usize,
    pub documents_ingested: usize,
    pub chunks_ingested: usize,
}

pub fn supported_source_kinds() -> &'static [&'static str] {
    &["markdown", "json", "seeded_catalog"]
}

pub fn default_corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("rag_corpus")
}

pub fn discover_markdown_sources(root: &Path) -> Result<Vec<IngestSource>, String> {
    let mut files = Vec::new();
    visit_markdown_files(root, &mut files)?;

    Ok(files
        .into_iter()
        .filter(|path| is_indexable_markdown(path))
        .map(|path| {
            let specialty = path
                .parent()
                .and_then(|p| p.file_name())
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            IngestSource {
                source_kind: "markdown".to_string(),
                source_path: path.to_string_lossy().to_string(),
                specialty,
            }
        })
        .collect())
}

fn visit_markdown_files(root: &Path, acc: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries =
        fs::read_dir(root).map_err(|e| format!("No se pudo leer corpus {}: {}", root.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("No se pudo leer entrada del corpus: {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            visit_markdown_files(&path, acc)?;
        } else if path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("md"))
            .unwrap_or(false)
        {
            acc.push(path);
        }
    }

    Ok(())
}

fn is_indexable_markdown(path: &Path) -> bool {
    fs::read_to_string(path)
        .ok()
        .and_then(|content| content.lines().next().map(|line| line.trim() == "---"))
        .unwrap_or(false)
}

pub fn parse_markdown_document(path: &Path) -> Result<IngestedDocument, String> {
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("No se pudo leer documento {}: {}", path.display(), e))?;
    let (frontmatter, body) = split_frontmatter(&raw)?;

    let title = frontmatter
        .get("title")
        .cloned()
        .ok_or_else(|| format!("Falta title en {}", path.display()))?;
    let slug = frontmatter
        .get("slug")
        .cloned()
        .ok_or_else(|| format!("Falta slug en {}", path.display()))?;
    let specialty = frontmatter
        .get("specialty")
        .cloned()
        .ok_or_else(|| format!("Falta specialty en {}", path.display()))?;
    let doc_type = frontmatter
        .get("doc_type")
        .cloned()
        .ok_or_else(|| format!("Falta doc_type en {}", path.display()))?;

    let entity_key = frontmatter.get("entity_key").cloned().filter(|v| !v.is_empty());
    let source_kind = frontmatter
        .get("source_kind")
        .cloned()
        .unwrap_or_else(|| "curated_markdown".to_string());
    let status = frontmatter
        .get("status")
        .cloned()
        .unwrap_or_else(|| "active".to_string());

    let summary = body
        .lines()
        .find(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .unwrap_or_default()
        .trim()
        .to_string();
    let content_hash = sha256_hex(&body);
    let chunks = chunk_markdown(&body, entity_key.clone());

    Ok(IngestedDocument {
        frontmatter: DocumentFrontmatter {
            title,
            slug,
            specialty,
            doc_type,
            entity_key,
            source_kind,
            status,
        },
        source_path: path.to_string_lossy().to_string(),
        body_markdown: body,
        summary,
        content_hash,
        chunks,
    })
}

fn split_frontmatter(raw: &str) -> Result<(HashMap<String, String>, String), String> {
    let mut lines = raw.lines();
    let first = lines.next().unwrap_or_default().trim();
    if first != "---" {
        return Err("Documento sin frontmatter YAML-like inicial".to_string());
    }

    let mut frontmatter = HashMap::new();
    let mut body_lines = Vec::new();
    let mut in_frontmatter = true;

    for line in lines {
        if in_frontmatter {
            if line.trim() == "---" {
                in_frontmatter = false;
                continue;
            }
            if let Some((k, v)) = line.split_once(':') {
                frontmatter.insert(k.trim().to_string(), v.trim().to_string());
            }
        } else {
            body_lines.push(line);
        }
    }

    if in_frontmatter {
        return Err("Frontmatter no cerrado correctamente".to_string());
    }

    Ok((frontmatter, body_lines.join("\n").trim().to_string()))
}

fn chunk_markdown(body: &str, entity_key: Option<String>) -> Vec<IngestedChunk> {
    let mut chunks = Vec::new();
    let mut current_anchor = String::from("overview");
    let mut current = String::new();

    for line in body.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('#') {
            if !current.trim().is_empty() {
                chunks.push(build_chunk(
                    chunks.len(),
                    &current,
                    &current_anchor,
                    entity_key.clone(),
                ));
                current.clear();
            }
            current_anchor = trimmed.trim_start_matches('#').trim().to_string();
            continue;
        }

        if current.len() + trimmed.len() + 1 > MAX_CHUNK_LEN && !current.trim().is_empty() {
            chunks.push(build_chunk(
                chunks.len(),
                &current,
                &current_anchor,
                entity_key.clone(),
            ));
            current.clear();
        }

        if !trimmed.is_empty() {
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(trimmed);
        }
    }

    if !current.trim().is_empty() {
        chunks.push(build_chunk(
            chunks.len(),
            &current,
            &current_anchor,
            entity_key,
        ));
    }

    chunks
}

fn build_chunk(
    chunk_index: usize,
    content: &str,
    title_anchor: &str,
    entity_key: Option<String>,
) -> IngestedChunk {
    IngestedChunk {
        chunk_index,
        chunk_text: content.trim().to_string(),
        title_anchor: title_anchor.to_string(),
        entity_key,
    }
}

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn embedding_payload(text: &str) -> Result<String, String> {
    serde_json::to_string(&generate_embedding(&normalize_text(text)))
        .map_err(|e| format!("No se pudo serializar embedding: {}", e))
}

fn specialty_id_from_code(code: &str) -> String {
    format!("sp_{}", code.trim().to_lowercase())
}

pub fn ingest_markdown_corpus(config: &RagConfig, root: &Path) -> Result<IngestReport, String> {
    let conn = ensure_database_ready(config)?;
    ingest_markdown_corpus_with_conn(&conn, root)
}

pub fn ingest_markdown_corpus_with_conn(
    conn: &Connection,
    root: &Path,
) -> Result<IngestReport, String> {
    let sources = discover_markdown_sources(root)?;
    let mut report = IngestReport {
        corpus_root: root.to_string_lossy().to_string(),
        files_discovered: sources.len(),
        ..Default::default()
    };

    for source in sources {
        let path = PathBuf::from(&source.source_path);
        let doc = parse_markdown_document(&path)?;
        persist_document(conn, &doc)?;
        report.documents_ingested += 1;
        report.chunks_ingested += doc.chunks.len();
    }

    Ok(report)
}

fn persist_document(conn: &Connection, doc: &IngestedDocument) -> Result<(), String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("No se pudo iniciar transaccion de ingesta: {}", e))?;

    let document_id = format!("doc_{}", Uuid::new_v4().simple());
    let specialty_id = specialty_id_from_code(&doc.frontmatter.specialty);
    let now = Utc::now().to_rfc3339();

    tx.execute(
        "DELETE FROM knowledge_chunk_embedding WHERE chunk_id IN (
            SELECT kc.id
            FROM knowledge_chunk kc
            INNER JOIN knowledge_document kd ON kd.id = kc.document_id
            WHERE kd.slug = ?1
        )",
        params![doc.frontmatter.slug],
    )
    .map_err(|e| format!("No se pudo limpiar embeddings previos: {}", e))?;

    tx.execute(
        "DELETE FROM knowledge_chunk_fts WHERE rowid IN (
            SELECT kc.rowid
            FROM knowledge_chunk kc
            INNER JOIN knowledge_document kd ON kd.id = kc.document_id
            WHERE kd.slug = ?1
        )",
        params![doc.frontmatter.slug],
    )
    .map_err(|e| format!("No se pudo limpiar FTS previo: {}", e))?;

    tx.execute(
        "DELETE FROM knowledge_chunk WHERE document_id IN (
            SELECT id FROM knowledge_document WHERE slug = ?1
        )",
        params![doc.frontmatter.slug],
    )
    .map_err(|e| format!("No se pudo limpiar chunks previos: {}", e))?;

    tx.execute(
        "DELETE FROM knowledge_document WHERE slug = ?1",
        params![doc.frontmatter.slug],
    )
    .map_err(|e| format!("No se pudo limpiar documento previo: {}", e))?;

    tx.execute(
        "INSERT INTO knowledge_document (
            id, specialty_id, doc_type, title, slug, summary, body_markdown,
            source_kind, source_path, version, status, content_hash, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, '1', ?10, ?11, ?12, ?12)",
        params![
            document_id,
            specialty_id,
            doc.frontmatter.doc_type,
            doc.frontmatter.title,
            doc.frontmatter.slug,
            doc.summary,
            doc.body_markdown,
            doc.frontmatter.source_kind,
            doc.source_path,
            doc.frontmatter.status,
            doc.content_hash,
            now
        ],
    )
    .map_err(|e| format!("No se pudo insertar knowledge_document: {}", e))?;

    for chunk in &doc.chunks {
        let chunk_id = format!("chunk_{}", Uuid::new_v4().simple());
        let embedding_json = embedding_payload(&format!(
            "{} {} {}",
            doc.frontmatter.title, chunk.title_anchor, chunk.chunk_text
        ))?;
        tx.execute(
            "INSERT INTO knowledge_chunk (
                id, document_id, chunk_index, chunk_text, specialty_id, entity_key,
                title_anchor, lexical_weight, semantic_weight, risk_level_hint
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1.0, 1.0, 'r0')",
            params![
                chunk_id,
                document_id,
                chunk.chunk_index as i64,
                chunk.chunk_text,
                specialty_id,
                chunk.entity_key,
                chunk.title_anchor
            ],
        )
        .map_err(|e| format!("No se pudo insertar knowledge_chunk: {}", e))?;

        let rowid: i64 = tx
            .query_row(
                "SELECT rowid FROM knowledge_chunk WHERE id = ?1",
                params![chunk_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("No se pudo resolver rowid del chunk: {}", e))?;

        tx.execute(
            "INSERT INTO knowledge_chunk_embedding (
                id, chunk_id, embedding_provider, embedding_model, embedding_json, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                format!("emb_{}", Uuid::new_v4().simple()),
                chunk_id,
                "kernelia_local",
                "hash64-windows-domain-v1",
                embedding_json,
                now
            ],
        )
        .map_err(|e| format!("No se pudo insertar knowledge_chunk_embedding: {}", e))?;

        tx.execute(
            "INSERT INTO knowledge_chunk_fts(rowid, chunk_text, title_anchor, entity_key)
             VALUES (?1, ?2, ?3, ?4)",
            params![rowid, chunk.chunk_text, chunk.title_anchor, chunk.entity_key],
        )
        .map_err(|e| format!("No se pudo insertar chunk en FTS: {}", e))?;
    }

    tx.commit()
        .map_err(|e| format!("No se pudo confirmar transaccion de ingesta: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag::models::RagConfig;
    use crate::rag::storage::sqlite::ensure_database_ready;

    #[test]
    fn parses_frontmatter_and_chunks_document() {
        let sample = r#"---
title: DNS Recovery Playbook
slug: network-dns-recovery
specialty: network
doc_type: playbook
entity_key: dns
source_kind: curated_markdown
status: active
---
# Contexto
El problema aplica cuando hay resolucion inconsistente.

# Verificacion
Validar gateway y ejecutar consulta DNS.
"#;

        let temp = std::env::temp_dir().join("kernelia_rag_ingest_test.md");
        fs::write(&temp, sample).expect("temp markdown");
        let doc = parse_markdown_document(&temp).expect("document parsed");
        assert_eq!(doc.frontmatter.specialty, "network");
        assert_eq!(doc.frontmatter.doc_type, "playbook");
        assert_eq!(doc.chunks.len(), 2);
        let _ = fs::remove_file(temp);
    }

    #[test]
    fn ingests_default_corpus_into_sqlite() {
        let config = RagConfig {
            enabled: false,
            db_filename: format!("kernelia_rag_test_{}.db", Uuid::new_v4().simple()),
            migrations_dir: "migrations".to_string(),
            seeds_dir: "seeds".to_string(),
        };
        let conn = ensure_database_ready(&config).expect("db ready");
        let report =
            ingest_markdown_corpus_with_conn(&conn, &default_corpus_root()).expect("ingested");
        assert!(report.documents_ingested >= 8);
        assert!(report.chunks_ingested >= report.documents_ingested);

        let embedding_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM knowledge_chunk_embedding", [], |row| row.get(0))
            .expect("embedding count");
        assert!(embedding_count >= report.chunks_ingested as i64);
    }
}
