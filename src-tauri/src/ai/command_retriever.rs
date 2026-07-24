use crate::rag::models::{DomainSpecialty, QueryAnalysis, RagConfig, RetrievalHit};
use crate::rag::retrieval::{
    final_score, sort_and_trim, specialty_code, specialty_id, RetrievalBundle,
};
use crate::rag::storage::sqlite::ensure_database_ready;
use rusqlite::{params, Connection};
#[cfg(test)]
use uuid::Uuid;

pub fn retrieve_commands(analysis: &QueryAnalysis) -> RetrievalBundle {
    let config = RagConfig::default();
    let Ok(conn) = ensure_database_ready(&config) else {
        return RetrievalBundle::default();
    };

    retrieve_commands_with_conn(&conn, analysis).unwrap_or_default()
}

pub fn retrieve_commands_with_conn(
    conn: &Connection,
    analysis: &QueryAnalysis,
) -> Result<RetrievalBundle, String> {
    let mut command_hits = query_command_hits(conn, analysis)?;
    let mut policy_hits = query_risk_hits(conn, analysis)?;

    sort_and_trim(&mut command_hits, 8);
    sort_and_trim(&mut policy_hits, 4);

    Ok(RetrievalBundle {
        command_hits,
        policy_hits,
        ..Default::default()
    })
}

fn query_command_hits(conn: &Connection, analysis: &QueryAnalysis) -> Result<Vec<RetrievalHit>, String> {
    let specialty_filter = specialty_id(&analysis.specialty);
    let like_query = format!("%{}%", analysis.normalized_text);
    let entity_filter = analysis.entities.first().cloned().unwrap_or_default();

    let mut stmt = conn
        .prepare(
            "SELECT
                wc.id,
                wc.canonical_name,
                wc.description,
                ds.code,
                wc.command_template,
                COALESCE(wca.alias_text, ''),
                COALESCE(tc.tool_name, '')
             FROM windows_command wc
             INNER JOIN domain_specialty ds ON ds.id = wc.specialty_id
             LEFT JOIN windows_command_alias wca ON wca.command_id = wc.id
             LEFT JOIN tool_command_binding tcb ON tcb.command_id = wc.id
             LEFT JOIN tool_capability tc ON tc.id = tcb.tool_id
             WHERE
                wc.canonical_name LIKE ?1
                OR wc.description LIKE ?1
                OR wc.command_template LIKE ?1
                OR COALESCE(wca.alias_text, '') LIKE ?1
                OR COALESCE(tc.tool_name, '') LIKE ?1
                OR (?2 <> '' AND COALESCE(wca.alias_text, '') LIKE '%' || ?2 || '%')
                OR (?3 <> 'sp_unknown' AND wc.specialty_id = ?3)",
        )
        .map_err(|e| format!("No se pudo preparar retrieval de comandos: {}", e))?;

    let rows = stmt
        .query_map(params![like_query, entity_filter, specialty_filter], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .map_err(|e| format!("No se pudo ejecutar retrieval de comandos: {}", e))?;

    let mut hits = Vec::new();
    for row in rows {
        let (id, canonical_name, description, specialty, command_template, alias_text, tool_name) =
            row.map_err(|e| format!("Fila invalida en retrieval de comandos: {}", e))?;
        let candidate_specialty = specialty_from_code(&specialty);
        let title = if tool_name.is_empty() {
            canonical_name.clone()
        } else {
            format!("{} -> {}", tool_name, canonical_name)
        };
        let content = format!(
            "{} {} {} {}",
            canonical_name, description, command_template, alias_text
        );
        let lexical = crate::rag::retrieval::lexical_score(&content, analysis);
        let exact = if alias_text == analysis.normalized_text || canonical_name == analysis.normalized_text {
            0.30
        } else {
            0.0
        };
        let score_final = (final_score(&content, &candidate_specialty, analysis) + exact).clamp(0.0, 1.4);

        if score_final <= 0.0 {
            continue;
        }

        hits.push(RetrievalHit {
            source_type: "command_or_tool".to_string(),
            source_id: id,
            title,
            score_lexical: lexical,
            score_vector: 0.0,
            score_final,
            specialty: candidate_specialty,
            entity_key: if alias_text.is_empty() { None } else { Some(alias_text) },
            content,
        });
    }

    Ok(hits)
}

fn query_risk_hits(conn: &Connection, analysis: &QueryAnalysis) -> Result<Vec<RetrievalHit>, String> {
    let specialty_code = specialty_code(&analysis.specialty);
    let like_query = format!("%{}%", analysis.normalized_text);

    let mut stmt = conn
        .prepare(
            "SELECT
                rp.id,
                rp.tool_name,
                rp.risk_level,
                rp.min_role,
                COALESCE(tc.specialty_id, ''),
                COALESCE(tc.description, '')
             FROM risk_policy rp
             LEFT JOIN tool_capability tc ON tc.tool_name = rp.tool_name
             WHERE
                rp.tool_name LIKE ?1
                OR COALESCE(tc.description, '') LIKE ?1
                OR COALESCE(tc.specialty_id, '') = ?2",
        )
        .map_err(|e| format!("No se pudo preparar retrieval de risk policy: {}", e))?;

    let rows = stmt
        .query_map(params![like_query, format!("sp_{}", specialty_code)], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })
        .map_err(|e| format!("No se pudo ejecutar retrieval de risk policy: {}", e))?;

    let mut hits = Vec::new();
    for row in rows {
        let (id, tool_name, risk_level, min_role, specialty_id, description) =
            row.map_err(|e| format!("Fila invalida en risk policy: {}", e))?;
        let candidate_specialty = specialty_from_specialty_id(&specialty_id);
        let content = format!(
            "tool_name={} risk_level={} min_role={} {}",
            tool_name, risk_level, min_role, description
        );
        let lexical = crate::rag::retrieval::lexical_score(&content, analysis);
        let score_final = (lexical
            + if candidate_specialty == analysis.specialty { 0.30 } else { 0.0 }
            + if analysis.normalized_text.contains(&tool_name.replace('_', " ")) {
                0.30
            } else {
                0.0
            })
            .clamp(0.0, 1.1);

        hits.push(RetrievalHit {
            source_type: "risk_policy".to_string(),
            source_id: id,
            title: format!("Risk policy {}", tool_name),
            score_lexical: lexical,
            score_vector: 0.0,
            score_final,
            specialty: candidate_specialty,
            entity_key: Some(risk_level),
            content,
        });
    }

    Ok(hits)
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

fn specialty_from_specialty_id(specialty_id: &str) -> DomainSpecialty {
    specialty_from_code(specialty_id.trim_start_matches("sp_"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::query_analyzer::analyze_query;

    fn test_conn() -> Connection {
        let config = RagConfig {
            enabled: false,
            db_filename: format!("kernelia_rag_commands_{}.db", Uuid::new_v4().simple()),
            migrations_dir: "migrations".to_string(),
            seeds_dir: "seeds".to_string(),
        };
        ensure_database_ready(&config).expect("db ready")
    }

    #[test]
    fn retrieves_dns_command_hits() {
        let conn = test_conn();
        let analysis = analyze_query("consulta dns");
        let bundle = retrieve_commands_with_conn(&conn, &analysis).expect("command bundle");

        assert!(!bundle.command_hits.is_empty());
        assert!(bundle
            .command_hits
            .iter()
            .any(|hit| hit.title.contains("dns_lookup") || hit.content.contains("nslookup")));
    }

    #[test]
    fn retrieves_service_risk_policy_for_restart_request() {
        let conn = test_conn();
        let analysis = analyze_query("reinicia el spooler");
        let bundle = retrieve_commands_with_conn(&conn, &analysis).expect("command bundle");

        assert!(bundle
            .policy_hits
            .iter()
            .any(|hit| hit.source_type == "risk_policy" && hit.specialty == DomainSpecialty::Services));
    }
}
