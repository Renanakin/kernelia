pub mod migrations;
pub mod sqlite;

use crate::rag::models::RagConfig;
use std::fs;
use std::path::PathBuf;

fn rag_dir() -> PathBuf {
    if let Ok(explicit_path) = std::env::var("KERNELIA_RAG_DB_PATH") {
        let path = PathBuf::from(explicit_path);
        if let Some(parent) = path.parent() {
            return parent.to_path_buf();
        }
    }

    if let Ok(explicit_dir) = std::env::var("KERNELIA_RAG_DIR") {
        let path = PathBuf::from(explicit_dir);
        if !path.as_os_str().is_empty() {
            return path;
        }
    }

    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("nexus-lite")
        .join("rag")
}

pub fn ensure_rag_dir() -> Result<PathBuf, String> {
    let dir = rag_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("No se pudo crear directorio RAG: {}", e))?;
    Ok(dir)
}

pub fn db_path(config: &RagConfig) -> Result<PathBuf, String> {
    if let Ok(explicit_path) = std::env::var("KERNELIA_RAG_DB_PATH") {
        let path = PathBuf::from(explicit_path);
        if !path.as_os_str().is_empty() {
            return Ok(path);
        }
    }

    Ok(ensure_rag_dir()?.join(&config.db_filename))
}

pub fn metadata_path() -> Result<PathBuf, String> {
    Ok(ensure_rag_dir()?.join("kernelia_rag_metadata.json"))
}

pub fn migration_names() -> Vec<&'static str> {
    migrations::all_migrations()
        .iter()
        .map(|(name, _)| *name)
        .collect()
}

pub fn migration_sql(name: &str) -> Option<&'static str> {
    migrations::all_migrations()
        .iter()
        .find_map(|(candidate, sql)| (*candidate == name).then_some(*sql))
}

pub fn base_seed_sql() -> &'static str {
    migrations::BASE_SEED_SQL
}

pub fn seed_names() -> Vec<&'static str> {
    migrations::all_seeds()
        .iter()
        .map(|(name, _)| *name)
        .collect()
}

pub fn seed_sql(name: &str) -> Option<&'static str> {
    migrations::all_seeds()
        .iter()
        .find_map(|(candidate, sql)| (*candidate == name).then_some(*sql))
}
