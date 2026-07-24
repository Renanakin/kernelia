use crate::rag::models::RagConfig;
use crate::rag::storage;
use rusqlite::Connection;

pub fn open_connection(config: &RagConfig) -> Result<Connection, String> {
    let path = storage::db_path(config)?;
    Connection::open(path).map_err(|e| format!("No se pudo abrir SQLite RAG: {}", e))
}

pub fn ensure_database_ready(config: &RagConfig) -> Result<Connection, String> {
    let conn = open_connection(config)?;

    for migration_name in storage::migration_names() {
        let sql = storage::migration_sql(migration_name)
            .ok_or_else(|| format!("Migracion no encontrada: {}", migration_name))?;
        conn.execute_batch(sql)
            .map_err(|e| format!("Error ejecutando migracion {}: {}", migration_name, e))?;
    }

    for seed_name in storage::seed_names() {
        let sql =
            storage::seed_sql(seed_name).ok_or_else(|| format!("Seed no encontrado: {}", seed_name))?;
        conn.execute_batch(sql)
            .map_err(|e| format!("Error ejecutando seed {}: {}", seed_name, e))?;
    }

    Ok(conn)
}
