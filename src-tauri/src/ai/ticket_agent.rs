use crate::rag::models::{RagConfig, SupportTicket, TicketCreationResult, TicketPriority};
use crate::rag::storage::sqlite::ensure_database_ready;
use rusqlite::params;
use uuid::Uuid;

pub fn infer_ticket_priority(query_text: &str) -> TicketPriority {
    let lowercase = query_text.to_lowercase();
    if lowercase.contains("pantalla azul")
        || lowercase.contains("bsod")
        || lowercase.contains("sin internet")
        || lowercase.contains("bloqueado")
        || lowercase.contains("caido")
    {
        TicketPriority::Alta
    } else if lowercase.contains("lento")
        || lowercase.contains("spooler")
        || lowercase.contains("impresora")
        || lowercase.contains("driver")
    {
        TicketPriority::Media
    } else {
        TicketPriority::Baja
    }
}

pub fn create_support_ticket_record(
    query_text: &str,
    specialty: &str,
    telemetry_json: &str,
) -> Result<TicketCreationResult, String> {
    let config = RagConfig::default();
    let conn = ensure_database_ready(&config)?;

    let now = chrono::Utc::now().to_rfc3339();
    let id = format!("ticket-{}", Uuid::new_v4());
    let ticket_number = rand_ticket_code();
    let priority = infer_ticket_priority(query_text);
    let priority_str = match priority {
        TicketPriority::Alta => "Alta",
        TicketPriority::Media => "Media",
        TicketPriority::Baja => "Baja",
    };

    let title = format!("Incidencia: {}", query_text.chars().take(50).collect::<String>());
    let description = format!(
        "Consulta no resuelta por RAG/Web: {}\nEspecialidad: {}\nTelemetría: {}",
        query_text, specialty, telemetry_json
    );

    conn.execute(
        "INSERT INTO support_ticket (id, ticket_code, title, description, priority, status, specialty, customer_id, telemetry_json, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'open', ?6, 'local_user', ?7, ?8, ?8)",
        params![id, ticket_number, title, description, priority_str, specialty, telemetry_json, now],
    ).map_err(|e| format!("Error guardando ticket en SQLite: {}", e))?;

    let customer_message = format!(
        "Se ha generado el ticket de soporte #{} con prioridad {}. Un especialista técnico revisará tu caso.",
        ticket_number, priority_str
    );

    Ok(TicketCreationResult {
        ticket_code: ticket_number,
        priority,
        description,
        customer_message,
    })
}

pub fn list_support_tickets_from_db() -> Result<Vec<SupportTicket>, String> {
    let config = RagConfig::default();
    let conn = ensure_database_ready(&config)?;

    let mut stmt = conn
        .prepare(
            "SELECT id, ticket_code, title, description, priority, status, specialty, customer_id, telemetry_json, created_at, updated_at
             FROM support_ticket ORDER BY created_at DESC LIMIT 50",
        )
        .map_err(|e| format!("Error preparando consulta de tickets: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(SupportTicket {
                id: row.get(0)?,
                ticket_code: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                priority: row.get(4)?,
                status: row.get(5)?,
                specialty: row.get(6)?,
                customer_id: row.get(7)?,
                telemetry_json: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(|e| format!("Error ejecutando consulta de tickets: {}", e))?;

    let mut tickets = Vec::new();
    for r in rows {
        if let Ok(t) = r {
            tickets.push(t);
        }
    }

    Ok(tickets)
}

fn rand_ticket_code() -> String {
    let now = chrono::Utc::now();
    let timestamp = now.format("%Y%m%d").to_string();
    let nanos = now.timestamp_nanos_opt().unwrap_or(12345).abs();
    let digits = (nanos % 9000) + 1000;
    format!("TK-{}-{}", timestamp, digits)
}
