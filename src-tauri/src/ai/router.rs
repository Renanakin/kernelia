use super::function_calling;
use super::confidence_engine;
use super::intent_engine;
use super::command_retriever;
use super::decision_engine;
use super::knowledge_retriever;
use super::live_state_retriever;
use super::memory_engine;
use super::prompt_context_builder;
use super::query_analyzer;
use super::specialty_agent;
use super::specialty_router;
use super::trace_engine;
use super::models::*;
use crate::config::AppSettings;
use crate::rag::models::{ConfidenceAssessment, DecisionEnvelope, QueryAnalysis};
use crate::rag::retrieval::RetrievalBundle;
use crate::tools::ToolEngine;
use crate::rag::{ConfidenceLevel, DecisionMode};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use uuid::Uuid;

/// System prompt del asistente KERNEL IA
const SYSTEM_PROMPT: &str = r#"Eres KERNEL IA, un sistema avanzado de diagnostico y asistencia tecnica especializado en Windows.
Tu objetivo es ayudar al usuario a identificar y solucionar problemas tecnicos, optimizar el rendimiento y mantener la seguridad del sistema.

Capacidades de diagnostico:
- Analisis de procesos, servicios y consumo de recursos (CPU/RAM/Disco).
- Verificacion de integridad de archivos y salud del sistema de archivos.
- Diagnostico de red (conectividad, latencia, DNS).
- Auditoria de seguridad basica y comprobacion de actualizaciones.

Directrices de interaccion:
1. Explica brevemente que vas a hacer y por que antes de ejecutar herramientas.
2. Si estas procesando datos en segundo plano, dilo de forma clara.
3. Usa tono tecnico pero accesible. Explica terminos complejos si es necesario.
4. No solicites contrasenas ni sugieras acciones inseguras.
5. Usa herramientas reales cuando el usuario pida diagnostico, red, procesos, archivos o reportes.

Actualmente operando en modo local/hibrido seguro.
Desarrollado por HackTeck SpA."#;
/// Router principal de IA: maneja la comunicaciÃƒÆ’Ã‚Â³n con los modelos
pub struct AiRouter {
    settings: Mutex<AppSettings>,
    history: Mutex<Vec<ChatMessage>>,
    rag_session_id: Mutex<Option<String>>,
}

struct GovernedContext {
    analysis: QueryAnalysis,
    retrieval: RetrievalBundle,
    confidence: ConfidenceAssessment,
    decision: DecisionEnvelope,
    live_state: live_state_retriever::LiveStateContext,
    prompt_context: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RagUiContext {
    pub enabled: bool,
    pub specialty: Option<String>,
    pub confidence_level: Option<String>,
    pub confidence_score: Option<f32>,
    pub decision_mode: Option<String>,
    pub risk_level: Option<String>,
    pub trace_id: Option<String>,
    pub show_summary_badge: bool,
    pub debug_panel_enabled: bool,
    pub retrieval_counts: Vec<String>,
    pub reason_codes: Vec<String>,
    pub live_conflicts: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RagComparisonContext {
    pub legacy_intent: String,
    pub legacy_confidence: f32,
    pub legacy_plan: Vec<String>,
    pub rag_specialty: String,
    pub rag_decision: String,
    pub rag_confidence: f32,
}

fn has_explicit_shortcut_intent(user_message: &str) -> bool {
    let text = normalize_query_text(user_message);
    text.starts_with("/tool")
        || text.starts_with("/quick")
        || text.contains("quick check")
        || text.contains("ejecuta")
        || text.contains("run ")
}

fn normalize_query_text(user_message: &str) -> String {
    let mut out = String::with_capacity(user_message.len());

    for ch in user_message.to_lowercase().chars() {
        let normalized = match ch {
            'á' | 'à' | 'ä' | 'â' | 'ã' | 'å' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' | 'õ' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            'ç' => 'c',
            c if c.is_ascii_alphanumeric() || c.is_whitespace() || c == '/' => c,
            _ => ' ',
        };
        out.push(normalized);
    }

    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn detect_fast_text_response(user_message: &str) -> Option<&'static str> {
    let normalized = normalize_query_text(user_message);

    if normalized == "ok"
        || normalized == "responde ok"
        || normalized == "responde solo ok"
        || normalized == "responde solamente ok"
        || normalized == "contesta ok"
        || normalized == "contesta solo ok"
    {
        return Some("ok");
    }

    None
}

fn detect_local_shortcut(
    user_message: &str,
) -> Option<(&'static str, serde_json::Value, &'static str, &'static str)> {
    let text = normalize_query_text(user_message);
    let explicit = has_explicit_shortcut_intent(user_message);

    if explicit
        && (text.contains("escritorio") || text.contains("desktop"))
        && (text.contains("lista") || text.contains("listar") || text.contains("archivos"))
    {
        return Some((
            "list_directory",
            serde_json::json!({ "path": "desktop" }),
            "Archivos del escritorio:\n\n",
            "No se pudo listar el escritorio. ",
        ));
    }

    if explicit
        && (text.contains("tarjeta")
            || text.contains("tarjetas")
            || text.contains("adaptador")
            || text.contains("adaptadores"))
        && text.contains("red")
    {
        return Some((
            "get_network_adapters",
            serde_json::json!({}),
            "Tarjetas/adaptadores de red detectados:\n\n",
            "No se pudo obtener la configuracion de adaptadores de red. ",
        ));
    }

    if explicit
        && (text.contains("diagnostico de red")
            || text.contains("conexion a internet")
            || text.contains("conexion internet"))
    {
        return Some((
            "run_network_diagnostic",
            serde_json::json!({}),
            "Diagnostico de red en tiempo real:\n\n",
            "No se pudo ejecutar el diagnostico de red. ",
        ));
    }

    if explicit
        && (text.contains("reporte tecnico")
            || text.contains("reporte técnico")
            || text.contains("genera un reporte"))
    {
        return Some((
            "generate_support_report",
            serde_json::json!({}),
            "Reporte tecnico generado:\n\n",
            "No se pudo generar el reporte tecnico. ",
        ));
    }

    if explicit
        && text.contains("procesos")
        && (text.contains("consumen") || text.contains("recursos") || text.contains("activos"))
    {
        return Some((
            "list_processes",
            serde_json::json!({ "sort_by": "memory", "limit": 15 }),
            "Procesos con mayor consumo:\n\n",
            "No se pudieron obtener los procesos. ",
        ));
    }

    if (text.contains("unidad")
        || text.contains("unidades")
        || text.contains("almacenamiento")
        || text.contains("disco")
        || text.contains("discos"))
        && (text.contains("cuanto")
            || text.contains("cuantos")
            || text.contains("tiene")
            || text.contains("lista")
            || text.contains("mostrar")
            || text.contains("muestr"))
    {
        return Some((
            "get_storage_summary",
            serde_json::json!({}),
            "",
            "No se pudo obtener informacion de almacenamiento. ",
        ));
    }

    if text.contains("estado actual del equipo")
        || text.contains("estado del equipo")
        || text.contains("estado del sistema")
    {
        return Some((
            "get_system_info",
            serde_json::json!({}),
            "Estado actual del equipo:\n\n",
            "No se pudo obtener el estado actual del equipo. ",
        ));
    }

    if text.contains("health")
        || text.contains("salud completa")
        || text.contains("salud del equipo")
        || text.contains("salud del sistema")
        || text.contains("health completo")
        || text.contains("reporte de salud")
    {
        return Some((
            "health_summary",
            serde_json::json!({}),
            "Health completo del equipo:\n\n",
            "No se pudo obtener el health del equipo. ",
        ));
    }

    None
}

#[derive(Debug, Clone)]
struct LocalToolRoute {
    tool_name: String,
    args: serde_json::Value,
    ok_prefix: &'static str,
    err_prefix: &'static str,
}

fn tool_definition_exists(tool_name: &str) -> bool {
    ToolEngine::get_tool_definitions()
        .iter()
        .any(|definition| definition.name == tool_name)
}

fn is_safe_local_first_tool(tool_name: &str) -> bool {
    matches!(
        tool_name,
        "get_system_info"
            | "get_storage_summary"
            | "list_processes"
            | "run_network_diagnostic"
            | "get_network_adapters"
            | "get_local_ip"
            | "get_default_gateway"
            | "get_dns_servers"
            | "get_wifi_info"
            | "get_public_ip"
            | "list_running_services"
            | "generate_support_report"
            | "health_overview"
            | "health_summary"
            | "scan_asset_inventory"
            | "list_driver_issues"
            | "search_missing_driver"
            | "list_directory"
            | "read_file"
            | "run_kernel_slowpc_diagnostic"
            | "run_kernel_network_playbook"
            | "generate_kernelia_readiness_report"
            | "generate_performance_report"
            | "generate_reliability_report"
            | "predict_operational_incidents"
            | "explain_root_cause"
            | "generate_autonomous_playbook"
            | "get_enterprise_dashboard"
            | "generate_advanced_reporting"
            | "get_noc_global_status"
    )
}

fn command_hit_supports_tool(governed: &GovernedContext, tool_name: &str) -> bool {
    governed.retrieval.command_hits.iter().any(|hit| {
        let title = hit.title.trim();
        title == tool_name
            || title.starts_with(&format!("{} -> ", tool_name))
            || hit.content.to_lowercase().contains(tool_name)
    })
}

fn local_route_for_tool(tool_name: &str) -> Option<LocalToolRoute> {
    match tool_name {
        "get_system_info" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Estado actual del equipo:\n\n",
            err_prefix: "No se pudo obtener el estado actual del equipo. ",
        }),
        "get_storage_summary" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Resumen de almacenamiento:\n\n",
            err_prefix: "No se pudo obtener informacion de almacenamiento. ",
        }),
        "list_processes" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({ "sort_by": "memory", "limit": 15 }),
            ok_prefix: "Procesos con mayor consumo:\n\n",
            err_prefix: "No se pudieron obtener los procesos. ",
        }),
        "run_network_diagnostic" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Diagnostico de red en tiempo real:\n\n",
            err_prefix: "No se pudo ejecutar el diagnostico de red. ",
        }),
        "get_network_adapters" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Tarjetas/adaptadores de red detectados:\n\n",
            err_prefix: "No se pudo obtener la configuracion de adaptadores de red. ",
        }),
        "get_local_ip" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "IP local detectada:\n\n",
            err_prefix: "No se pudo obtener la IP local. ",
        }),
        "get_default_gateway" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Gateway por defecto:\n\n",
            err_prefix: "No se pudo obtener el gateway por defecto. ",
        }),
        "get_dns_servers" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Servidores DNS detectados:\n\n",
            err_prefix: "No se pudo obtener la lista de DNS. ",
        }),
        "get_wifi_info" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Estado de Wi-Fi:\n\n",
            err_prefix: "No se pudo obtener informacion de Wi-Fi. ",
        }),
        "get_public_ip" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "IP publica actual:\n\n",
            err_prefix: "No se pudo obtener la IP publica. ",
        }),
        "get_windows_updates_status" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Estado de Windows Update:\n\n",
            err_prefix: "No se pudo obtener el estado de Windows Update. ",
        }),
        "check_app_updates" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Actualizaciones de aplicaciones:\n\n",
            err_prefix: "No se pudieron listar las actualizaciones de aplicaciones. ",
        }),
        "list_running_services" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Servicios en ejecucion:\n\n",
            err_prefix: "No se pudieron obtener los servicios en ejecucion. ",
        }),
        "generate_support_report" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Reporte tecnico generado:\n\n",
            err_prefix: "No se pudo generar el reporte tecnico. ",
        }),
        "health_overview" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Health general del equipo:\n\n",
            err_prefix: "No se pudo obtener el health general del equipo. ",
        }),
        "health_summary" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Health completo del equipo:\n\n",
            err_prefix: "No se pudo obtener el health del equipo. ",
        }),
        "scan_asset_inventory" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Inventario de activos:\n\n",
            err_prefix: "No se pudo generar el inventario de activos. ",
        }),
        "list_driver_issues" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Controladores con problema:\n\n",
            err_prefix: "No se pudieron obtener los problemas de controladores. ",
        }),
        "search_missing_driver" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Busqueda de controlador faltante:\n\n",
            err_prefix: "No se pudo buscar el controlador faltante. ",
        }),
        "list_directory" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({ "path": "desktop" }),
            ok_prefix: "Archivos del escritorio:\n\n",
            err_prefix: "No se pudo listar el escritorio. ",
        }),
        "read_file" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({ "path": "" }),
            ok_prefix: "Contenido del archivo:\n\n",
            err_prefix: "No se pudo leer el archivo. ",
        }),
        "run_kernel_slowpc_diagnostic" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Diagnostico KernelIA para equipo lento:\n\n",
            err_prefix: "No se pudo ejecutar el diagnostico de equipo lento. ",
        }),
        "run_kernel_network_playbook" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Playbook KernelIA de red:\n\n",
            err_prefix: "No se pudo ejecutar el playbook de red. ",
        }),
        "generate_kernelia_readiness_report" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Reporte de readiness KernelIA:\n\n",
            err_prefix: "No se pudo generar el reporte de readiness. ",
        }),
        "generate_performance_report" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Reporte de rendimiento:\n\n",
            err_prefix: "No se pudo generar el reporte de rendimiento. ",
        }),
        "generate_reliability_report" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Reporte de fiabilidad:\n\n",
            err_prefix: "No se pudo generar el reporte de fiabilidad. ",
        }),
        "predict_operational_incidents" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Prediccion de incidentes operacionales:\n\n",
            err_prefix: "No se pudo calcular la prediccion de incidentes. ",
        }),
        "explain_root_cause" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Causa raiz probable:\n\n",
            err_prefix: "No se pudo explicar la causa raiz. ",
        }),
        "generate_autonomous_playbook" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Playbook autonomo generado:\n\n",
            err_prefix: "No se pudo generar el playbook autonomo. ",
        }),
        "get_enterprise_dashboard" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Dashboard enterprise:\n\n",
            err_prefix: "No se pudo obtener el dashboard enterprise. ",
        }),
        "generate_advanced_reporting" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Reporte avanzado generado:\n\n",
            err_prefix: "No se pudo generar el reporte avanzado. ",
        }),
        "get_noc_global_status" => Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args: serde_json::json!({}),
            ok_prefix: "Estado NOC global:\n\n",
            err_prefix: "No se pudo obtener el estado NOC global. ",
        }),
        _ => None,
    }
}

fn keyword_route_for_analysis(analysis: &QueryAnalysis, normalized: &str) -> Option<LocalToolRoute> {
    let has_count_or_listing_intent = normalized.contains("cuanto")
        || normalized.contains("cuantos")
        || normalized.contains("cuantas")
        || normalized.contains("listar")
        || normalized.contains("lista")
        || normalized.contains("mostrar")
        || normalized.contains("muestr")
        || normalized.contains("tiene");

    if analysis.specialty == crate::rag::models::DomainSpecialty::Network
        || normalized.contains("internet")
        || normalized.contains("red")
        || normalized.contains("dns")
        || normalized.contains("gateway")
        || normalized.contains("wifi")
        || normalized.contains("latencia")
    {
        return local_route_for_tool("run_network_diagnostic");
    }

    if normalized.contains("proceso")
        || normalized.contains("procesos")
        || normalized.contains("cpu")
        || normalized.contains("memoria")
        || normalized.contains("recursos")
        || normalized.contains("consumen")
        || normalized.contains("consume")
    {
        return local_route_for_tool("list_processes");
    }

    if normalized.contains("spooler")
        || normalized.contains("servicio")
        || normalized.contains("services")
        || normalized.contains("service")
    {
        return local_route_for_tool("list_running_services");
    }

    if normalized.contains("windows update")
        || normalized.contains("estado de actualizaciones")
        || normalized.contains("estado de windows update")
        || normalized.contains("actualizaciones de windows")
        || normalized.contains("lista de actualizaciones")
        || normalized.contains("ver actualizaciones")
    {
        return local_route_for_tool("get_windows_updates_status");
    }

    if normalized.contains("actualizaciones de apps")
        || normalized.contains("actualizaciones de aplicaciones")
        || normalized.contains("actualizaciones disponibles")
        || normalized.contains("winget upgrade")
    {
        return local_route_for_tool("check_app_updates");
    }

    if normalized.contains("driver")
        || normalized.contains("controlador")
        || normalized.contains("codigo 43")
        || normalized.contains("code 43")
        || normalized.contains("gpu")
        || normalized.contains("audio")
        || normalized.contains("usb")
    {
        return local_route_for_tool("list_driver_issues");
    }

    if normalized.contains("escritorio")
        || normalized.contains("desktop")
        || normalized.contains("archivo")
        || normalized.contains("carpeta")
        || normalized.contains("ruta")
    {
        return local_route_for_tool("list_directory");
    }

    if normalized.contains("almacenamiento")
        || normalized.contains("disco")
        || normalized.contains("discos")
        || normalized.contains("ssd")
        || normalized.contains("hdd")
        || normalized.contains("volumen")
    {
        if has_count_or_listing_intent {
            return local_route_for_tool("get_storage_summary");
        }
    }

    if normalized.contains("health")
        || normalized.contains("salud")
        || normalized.contains("estado")
        || normalized.contains("sistema")
        || normalized.contains("equipo")
        || normalized.contains("pc")
        || normalized.contains("lento")
    {
        if normalized.contains("disco")
            || normalized.contains("almacenamiento")
            || normalized.contains("volumen")
        {
            return local_route_for_tool("get_storage_summary");
        }
        return local_route_for_tool("health_summary");
    }

    if analysis.specialty == crate::rag::models::DomainSpecialty::System
        || analysis.specialty == crate::rag::models::DomainSpecialty::Telemetry
        || analysis.specialty == crate::rag::models::DomainSpecialty::Maintenance
        || analysis.specialty == crate::rag::models::DomainSpecialty::Software
    {
        return local_route_for_tool("health_summary");
    }

    None
}

fn retrieval_route_for_governed(governed: &GovernedContext) -> Option<LocalToolRoute> {
    let mut candidates: Vec<String> =
        specialty_agent::preferred_tools_for_message(&governed.analysis.normalized_text)
            .into_iter()
            .map(|name| name.to_string())
            .collect();

    candidates.extend(
        governed
            .retrieval
            .command_hits
            .iter()
            .filter_map(|hit| hit.title.split(" -> ").next().map(|value| value.trim().to_string())),
    );

    let best = candidates
        .into_iter()
        .find(|tool_name| {
            let tool_name = tool_name.as_str();
            is_safe_local_first_tool(tool_name)
                && tool_definition_exists(tool_name)
                && (command_hit_supports_tool(governed, tool_name)
                    || governed
                        .decision
                        .allowed_tools
                        .iter()
                        .any(|allowed| allowed == tool_name))
        })?;

    local_route_for_tool(best.as_str())
}

fn resolve_local_first_route(user_message: &str, governed: &GovernedContext) -> Option<LocalToolRoute> {
    if governed.decision.requires_clarification
        || governed.decision.requires_human
        || matches!(
            governed.decision.decision_mode,
            DecisionMode::Clarify | DecisionMode::Deny
        )
        || matches!(governed.decision.confidence_level, ConfidenceLevel::Low)
    {
        return None;
    }

    let normalized = normalize_query_text(user_message);

    if let Some((tool_name, args, ok_prefix, err_prefix)) = detect_local_shortcut(user_message) {
        return Some(LocalToolRoute {
            tool_name: tool_name.to_string(),
            args,
            ok_prefix,
            err_prefix,
        });
    }

    if let Some(route) = keyword_route_for_analysis(&governed.analysis, &normalized) {
        return Some(route);
    }

    retrieval_route_for_governed(governed)
}
impl AiRouter {
    fn build_system_context_message(
        &self,
        user_message: &str,
        governed: Option<&GovernedContext>,
    ) -> ChatMessage {
        let recent = self.recent_context_window(6);
        let op_analysis = intent_engine::analyze_message(user_message, &recent);
        let operational_context = intent_engine::to_operational_context(&op_analysis);
        let content = if let Some(governed) = governed {
            format!("{}\n\n{}", operational_context, governed.prompt_context)
        } else {
            operational_context
        };

        ChatMessage {
            role: MessageRole::System,
            content,
            reasoning_content: None,
            tool_call_id: None,
            tool_calls: None,
        }
    }

    fn build_rag_ui_context(
        &self,
        governed: Option<&GovernedContext>,
        settings: &AppSettings,
        trace_id: Option<String>,
    ) -> RagUiContext {
        if let Some(governed) = governed {
            return RagUiContext {
                enabled: true,
                specialty: Some(format!("{:?}", governed.decision.specialty).to_lowercase()),
                confidence_level: Some(
                    format!("{:?}", governed.decision.confidence_level).to_lowercase(),
                ),
                confidence_score: Some(governed.decision.confidence_score),
                decision_mode: Some(format!("{:?}", governed.decision.decision_mode).to_lowercase()),
                risk_level: Some(format!("{:?}", governed.decision.risk_level).to_lowercase()),
                trace_id,
                show_summary_badge: settings.rag_show_confidence_badge,
                debug_panel_enabled: settings.rag_debug_panel,
                retrieval_counts: vec![
                    format!("knowledge={}", governed.retrieval.knowledge_hits.len()),
                    format!("commands={}", governed.retrieval.command_hits.len()),
                    format!("policies={}", governed.retrieval.policy_hits.len()),
                ],
                reason_codes: governed.decision.reason_codes.clone(),
                live_conflicts: governed.live_state.conflict_flags.clone(),
            };
        }

        RagUiContext {
            enabled: false,
            specialty: None,
            confidence_level: None,
            confidence_score: None,
            decision_mode: None,
            risk_level: None,
            trace_id,
            show_summary_badge: false,
            debug_panel_enabled: settings.rag_debug_panel,
            retrieval_counts: vec!["legacy_mode".to_string()],
            reason_codes: vec!["RAG_DISABLED".to_string()],
            live_conflicts: Vec::new(),
        }
    }

    fn build_rag_comparison_context(
        &self,
        user_message: &str,
        governed: Option<&GovernedContext>,
        settings: &AppSettings,
    ) -> Option<RagComparisonContext> {
        if !settings.rag_compare_mode || !settings.rag_engine_enabled {
            return None;
        }

        let governed = governed?;
        let recent = self.recent_context_window(6);
        let legacy = intent_engine::analyze_message(user_message, &recent);

        Some(RagComparisonContext {
            legacy_intent: format!("{:?}", legacy.intent).to_lowercase(),
            legacy_confidence: legacy.confidence,
            legacy_plan: legacy.recommended_plan.clone(),
            rag_specialty: format!("{:?}", governed.decision.specialty).to_lowercase(),
            rag_decision: format!("{:?}", governed.decision.decision_mode).to_lowercase(),
            rag_confidence: governed.decision.confidence_score,
        })
    }

    fn recent_context_window(&self, max_items: usize) -> Vec<String> {
        let Ok(history) = self.history.lock() else {
            return Vec::new();
        };
        history
            .iter()
            .rev()
            .filter(|m| m.role == MessageRole::User || m.role == MessageRole::Assistant)
            .take(max_items)
            .map(|m| m.content.clone())
            .collect()
    }

    fn get_or_create_rag_session_id(&self) -> Result<String, String> {
        let mut guard = self.rag_session_id.lock().map_err(|e| e.to_string())?;
        if let Some(session_id) = guard.clone() {
            return Ok(session_id);
        }

        let session_id = memory_engine::create_session()?;
        *guard = Some(session_id.clone());
        Ok(session_id)
    }

    fn build_governed_context(&self, user_message: &str) -> Result<GovernedContext, String> {
        let rag_analysis = query_analyzer::analyze_query(user_message);
        let routed_specialty = specialty_router::route_specialty(&rag_analysis);
        let mut routed_analysis = rag_analysis.clone();
        routed_analysis.specialty = routed_specialty;

        let knowledge_bundle = knowledge_retriever::retrieve_knowledge(&routed_analysis);
        let command_bundle = command_retriever::retrieve_commands(&routed_analysis);
        let retrieval = crate::rag::retrieval::RetrievalBundle {
            knowledge_hits: knowledge_bundle.knowledge_hits,
            command_hits: command_bundle.command_hits,
            policy_hits: knowledge_bundle
                .policy_hits
                .into_iter()
                .chain(command_bundle.policy_hits)
                .collect(),
            memory_hits: Vec::new(),
        };
        let confidence = confidence_engine::assess_confidence(&routed_analysis, &retrieval);
        let decision = decision_engine::build_decision(&routed_analysis, &retrieval, &confidence);
        let live_state = if decision.requires_live_state {
            live_state_retriever::retrieve_live_state(decision.specialty.clone())
        } else {
            live_state_retriever::LiveStateContext::default()
        };
        let agent_profile = specialty_agent::build_specialty_agent_profile(&decision.specialty);
        let session_id = self.get_or_create_rag_session_id()?;
        let memory_summary = memory_engine::load_latest_memory_summary(&session_id)?;
        let prompt_context = prompt_context_builder::build_prompt_context(
            &routed_analysis,
            &decision,
            &retrieval,
            &live_state,
            &agent_profile,
            memory_summary.as_deref(),
        );

        Ok(GovernedContext {
            analysis: routed_analysis,
            retrieval,
            confidence,
            decision,
            live_state,
            prompt_context,
        })
    }

    async fn try_local_shortcut(
        &self,
        app: &tauri::AppHandle,
        user_message: &str,
    ) -> Option<(ChatResponse, String)> {
        let (tool_name, args, ok_prefix, err_prefix) = detect_local_shortcut(user_message)?;

        let role = {
            let settings = self.settings.lock().ok()?;
            settings.user_role
        };
        let result = ToolEngine::execute(app, tool_name, &args, role).await;

        let response_text = if result.success {
            format!("{}{}", ok_prefix, result.output)
        } else {
            format!(
                "{}{}",
                err_prefix,
                result
                    .error
                    .unwrap_or_else(|| "Error desconocido.".to_string())
            )
        };

        if let Ok(mut history) = self.history.lock() {
            history.push(ChatMessage {
                role: MessageRole::Assistant,
                content: response_text.clone(),
                reasoning_content: None,
                tool_call_id: None,
                tool_calls: None,
            });
        }

        Some((
            ChatResponse {
                text: response_text,
                tools_used: vec![ToolUseInfo {
                    name: tool_name.to_string(),
                    arguments: args.to_string(),
                }],
                model: "local-tools".to_string(),
                rag_context: None,
                rag_comparison: None,
                error: None,
            },
            tool_name.to_string(),
        ))
    }

    async fn execute_local_route(
        &self,
        app: &tauri::AppHandle,
        user_message: &str,
        settings: &AppSettings,
        governed: Option<&GovernedContext>,
        started_at: Instant,
        route: LocalToolRoute,
    ) -> Result<ChatResponse, String> {
        let role = {
            let settings = self.settings.lock().map_err(|e| e.to_string())?;
            settings.user_role
        };

        let result = ToolEngine::execute(app, route.tool_name.as_str(), &route.args, role).await;
        let response_text = if result.success {
            format!("{}{}", route.ok_prefix, result.output)
        } else {
            format!(
                "{}{}",
                route.err_prefix,
                result
                    .error
                    .unwrap_or_else(|| "Error desconocido.".to_string())
            )
        };

        if let Ok(mut history) = self.history.lock() {
            history.push(ChatMessage {
                role: MessageRole::Assistant,
                content: response_text.clone(),
                reasoning_content: None,
                tool_call_id: None,
                tool_calls: None,
            });
        }

        let trace_tools_used = vec![function_calling::ToolInfo {
            name: route.tool_name.to_string(),
            arguments: route.args.to_string(),
        }];
        let tools_used = vec![ToolUseInfo {
            name: route.tool_name.clone(),
            arguments: route.args.to_string(),
        }];

        let mut trace_id = None;
        if let Some(governed) = governed {
            let session_id = self.get_or_create_rag_session_id()?;
            let current_trace_id = format!("trace_{}", Uuid::new_v4().simple());
            let trace = trace_engine::build_trace(
                &current_trace_id,
                Some(&session_id),
                user_message,
                &governed.analysis,
                &governed.retrieval,
                &governed.confidence,
                &governed.decision,
                started_at.elapsed().as_millis() as i64,
            );
            let _ = trace_engine::persist_trace(&trace, &trace_tools_used);
            let _ = memory_engine::persist_session_memory(
                &session_id,
                user_message,
                &response_text,
                &governed.analysis,
                &governed.decision,
                &governed.live_state,
                &trace_tools_used,
            );
            trace_id = Some(current_trace_id);
        }

        Ok(ChatResponse {
            text: response_text,
            tools_used,
            model: "local-tools".to_string(),
            rag_context: Some(self.build_rag_ui_context(governed, settings, trace_id)),
            rag_comparison: self.build_rag_comparison_context(user_message, governed, settings),
            error: None,
        })
    }
    /// Crea una nueva instancia del router
    pub fn new(settings: AppSettings) -> Self {
        let mut history = vec![ChatMessage {
            role: MessageRole::System,
            content: SYSTEM_PROMPT.to_string(),
            reasoning_content: None,
            tool_call_id: None,
            tool_calls: None,
        }];

        // Si es primer inicio, agregar mensaje de bienvenida
        if settings.first_run {
            history.push(ChatMessage {
                role: MessageRole::System,
                content: "Este es el primer inicio del usuario. PresÃƒÂ©ntate brevemente y ofrece ayuda para configurar las API keys de los modelos de IA.".to_string(),
            reasoning_content: None,
                tool_call_id: None,
                tool_calls: None,
            });
        }

        Self {
            settings: Mutex::new(settings),
            history: Mutex::new(history),
            rag_session_id: Mutex::new(None),
        }
    }

    /// EnvÃƒÂ­a un mensaje del usuario y obtiene la respuesta del modelo
    pub async fn send_message(
        &self,
        app: &tauri::AppHandle,
        user_message: &str,
    ) -> Result<ChatResponse, String> {
        let started_at = Instant::now();
        // Agregar mensaje del usuario al historial
        {
            let mut history = self.history.lock().map_err(|e| e.to_string())?;
            history.push(ChatMessage {
                role: MessageRole::User,
                content: user_message.to_string(),
                reasoning_content: None,
                tool_call_id: None,
                tool_calls: None,
            });
        }

        let settings = {
            let s = self.settings.lock().map_err(|e| e.to_string())?;
            s.clone()
        };

        if let Some(text) = detect_fast_text_response(user_message) {
            if let Ok(mut history) = self.history.lock() {
                history.push(ChatMessage {
                    role: MessageRole::Assistant,
                    content: text.to_string(),
                    reasoning_content: None,
                    tool_call_id: None,
                    tool_calls: None,
                });
            }
            return Ok(ChatResponse {
                text: text.to_string(),
                tools_used: vec![],
                model: "kernel-fastpath".to_string(),
                rag_context: None,
                rag_comparison: None,
                error: None,
            });
        }

        if let Some((shortcut_response, _tool_name)) =
            self.try_local_shortcut(app, user_message).await
        {
            return Ok(shortcut_response);
        }

        let governed = Some(self.build_governed_context(user_message)?);

        if let Some(route) = governed
            .as_ref()
            .and_then(|context| resolve_local_first_route(user_message, context))
        {
            return self
                .execute_local_route(
                    app,
                    user_message,
                    &settings,
                    governed.as_ref(),
                    started_at,
                    route,
                )
                .await;
        }

        let mut messages = {
            let h = self.history.lock().map_err(|e| e.to_string())?;
            h.clone()
        };
        messages.push(self.build_system_context_message(user_message, governed.as_ref()));

        // Ejecutar el loop de function calling
        let result = function_calling::function_calling_loop(app, &mut messages, &settings).await;

        // Actualizar historial con los mensajes resultantes
        {
            let mut history = self.history.lock().map_err(|e| e.to_string())?;
            *history = messages;

            // Truncar historial si es muy largo
            let max = settings.max_history_messages;
            if history.len() > max {
                // Mantener el system prompt + los ÃƒÂºltimos N mensajes
                let system_msgs: Vec<_> = history
                    .iter()
                    .take_while(|m| m.role == MessageRole::System)
                    .cloned()
                    .collect();
                let rest: Vec<_> = history.iter().skip(system_msgs.len()).cloned().collect();
                let keep = rest.len().saturating_sub(max - system_msgs.len());
                let mut new_history = system_msgs;
                new_history.extend(rest.into_iter().skip(keep));
                *history = new_history;
            }
        }

        match result {
            Ok(fc_result) => {
                let assistant_response = fc_result.response.clone();
                let mut trace_id = None;
                if let Some(governed) = governed.as_ref() {
                    let session_id = self.get_or_create_rag_session_id()?;
                    let current_trace_id = format!("trace_{}", Uuid::new_v4().simple());
                    let trace = trace_engine::build_trace(
                        &current_trace_id,
                        Some(&session_id),
                        user_message,
                        &governed.analysis,
                        &governed.retrieval,
                        &governed.confidence,
                        &governed.decision,
                        started_at.elapsed().as_millis() as i64,
                    );
                    let _ = trace_engine::persist_trace(&trace, &fc_result.tools_used);
                    let _ = memory_engine::persist_session_memory(
                        &session_id,
                        user_message,
                        &assistant_response,
                        &governed.analysis,
                        &governed.decision,
                        &governed.live_state,
                        &fc_result.tools_used,
                    );
                    trace_id = Some(current_trace_id);
                }
                let tools_used: Vec<ToolUseInfo> = fc_result
                    .tools_used
                    .into_iter()
                    .map(|t| ToolUseInfo {
                        name: t.name,
                        arguments: t.arguments,
                    })
                    .collect();
                Ok(ChatResponse {
                    text: assistant_response,
                    tools_used,
                    model: settings.selected_model.clone(),
                    rag_context: Some(self.build_rag_ui_context(
                        governed.as_ref(),
                        &settings,
                        trace_id,
                    )),
                    rag_comparison: self.build_rag_comparison_context(
                        user_message,
                        governed.as_ref(),
                        &settings,
                    ),
                    error: None,
                })
            }
            Err(e) => {
                let mut trace_id = None;
                if let Some(governed) = governed.as_ref() {
                    let session_id = self.get_or_create_rag_session_id()?;
                    let current_trace_id = format!("trace_{}", Uuid::new_v4().simple());
                    let mut trace = trace_engine::build_trace(
                        &current_trace_id,
                        Some(&session_id),
                        user_message,
                        &governed.analysis,
                        &governed.retrieval,
                        &governed.confidence,
                        &governed.decision,
                        started_at.elapsed().as_millis() as i64,
                    );
                    trace_engine::append_error(
                        &mut trace,
                        "function_calling",
                        "LLM_RESPONSE_ERROR",
                        &e,
                    );
                    let _ = trace_engine::persist_trace(&trace, &[]);
                    trace_id = Some(current_trace_id);
                }
                Ok(ChatResponse {
                    text: String::new(),
                    tools_used: vec![],
                    model: settings.selected_model.clone(),
                    rag_context: Some(self.build_rag_ui_context(
                        governed.as_ref(),
                        &settings,
                        trace_id,
                    )),
                    rag_comparison: self.build_rag_comparison_context(
                        user_message,
                        governed.as_ref(),
                        &settings,
                    ),
                    error: Some(e),
                })
            }
        }
    }

    /// EnvÃƒÂ­a un mensaje del usuario y obtiene la respuesta del modelo en streaming
    pub async fn stream_message(
        &self,
        app: &tauri::AppHandle,
        user_message: &str,
        on_update: Arc<dyn Fn(StreamUpdate) + Send + Sync + 'static>,
    ) -> Result<ChatResponse, String> {
        let started_at = Instant::now();
        // Agregar mensaje del usuario al historial
        {
            let mut history = self.history.lock().map_err(|e| e.to_string())?;
            history.push(ChatMessage {
                role: MessageRole::User,
                content: user_message.to_string(),
                reasoning_content: None,
                tool_call_id: None,
                tool_calls: None,
            });
        }

        let settings = {
            let s = self.settings.lock().map_err(|e| e.to_string())?;
            s.clone()
        };

        if let Some(text) = detect_fast_text_response(user_message) {
            on_update(StreamUpdate {
                update_type: "text".to_string(),
                content: text.to_string(),
                tool_name: None,
                tool_result: None,
            });
            if let Ok(mut history) = self.history.lock() {
                history.push(ChatMessage {
                    role: MessageRole::Assistant,
                    content: text.to_string(),
                    reasoning_content: None,
                    tool_call_id: None,
                    tool_calls: None,
                });
            }
            return Ok(ChatResponse {
                text: text.to_string(),
                tools_used: vec![],
                model: "kernel-fastpath".to_string(),
                rag_context: None,
                rag_comparison: None,
                error: None,
            });
        }

        if let Some((shortcut_response, tool_name)) =
            self.try_local_shortcut(app, user_message).await
        {
            on_update(StreamUpdate {
                update_type: "tool_start".to_string(),
                content: String::new(),
                tool_name: Some(tool_name.clone()),
                tool_result: None,
            });
            on_update(StreamUpdate {
                update_type: "tool_end".to_string(),
                content: String::new(),
                tool_name: Some(tool_name),
                tool_result: Some("OK".to_string()),
            });
            on_update(StreamUpdate {
                update_type: "text".to_string(),
                content: shortcut_response.text.clone(),
                tool_name: None,
                tool_result: None,
            });
            return Ok(shortcut_response);
        }

        let governed = Some(self.build_governed_context(user_message)?);

        if let Some(route) = governed
            .as_ref()
            .and_then(|context| resolve_local_first_route(user_message, context))
        {
            on_update(StreamUpdate {
                update_type: "tool_start".to_string(),
                content: String::new(),
                tool_name: Some(route.tool_name.to_string()),
                tool_result: None,
            });
            let local_response = self
                .execute_local_route(
                    app,
                    user_message,
                    &settings,
                    governed.as_ref(),
                    started_at,
                    route,
                )
                .await?;
            on_update(StreamUpdate {
                update_type: "tool_end".to_string(),
                content: String::new(),
                tool_name: local_response.tools_used.first().map(|tool| tool.name.clone()),
                tool_result: Some("OK".to_string()),
            });
            on_update(StreamUpdate {
                update_type: "text".to_string(),
                content: local_response.text.clone(),
                tool_name: None,
                tool_result: None,
            });
            return Ok(local_response);
        }

        let mut messages = {
            let h = self.history.lock().map_err(|e| e.to_string())?;
            h.clone()
        };
        messages.push(self.build_system_context_message(user_message, governed.as_ref()));

        // Ejecutar el loop de function calling en modo streaming
        let result = function_calling::stream_function_calling_loop(
            app,
            &mut messages,
            &settings,
            on_update,
        )
        .await;

        // Actualizar historial con los mensajes resultantes
        {
            let mut history = self.history.lock().map_err(|e| e.to_string())?;
            *history = messages;

            // Truncar historial si es muy largo
            let max = settings.max_history_messages;
            if history.len() > max {
                let system_msgs: Vec<_> = history
                    .iter()
                    .take_while(|m| m.role == MessageRole::System)
                    .cloned()
                    .collect();
                let rest: Vec<_> = history.iter().skip(system_msgs.len()).cloned().collect();
                let keep = rest.len().saturating_sub(max - system_msgs.len());
                let mut new_history = system_msgs;
                new_history.extend(rest.into_iter().skip(keep));
                *history = new_history;
            }
        }

        match result {
            Ok(fc_result) => {
                let assistant_response = fc_result.response.clone();
                let mut trace_id = None;
                if let Some(governed) = governed.as_ref() {
                    let session_id = self.get_or_create_rag_session_id()?;
                    let current_trace_id = format!("trace_{}", Uuid::new_v4().simple());
                    let trace = trace_engine::build_trace(
                        &current_trace_id,
                        Some(&session_id),
                        user_message,
                        &governed.analysis,
                        &governed.retrieval,
                        &governed.confidence,
                        &governed.decision,
                        started_at.elapsed().as_millis() as i64,
                    );
                    let _ = trace_engine::persist_trace(&trace, &fc_result.tools_used);
                    let _ = memory_engine::persist_session_memory(
                        &session_id,
                        user_message,
                        &assistant_response,
                        &governed.analysis,
                        &governed.decision,
                        &governed.live_state,
                        &fc_result.tools_used,
                    );
                    trace_id = Some(current_trace_id);
                }
                let tools_used: Vec<ToolUseInfo> = fc_result
                    .tools_used
                    .into_iter()
                    .map(|t| ToolUseInfo {
                        name: t.name,
                        arguments: t.arguments,
                    })
                    .collect();
                Ok(ChatResponse {
                    text: assistant_response,
                    tools_used,
                    model: settings.selected_model.clone(),
                    rag_context: Some(self.build_rag_ui_context(
                        governed.as_ref(),
                        &settings,
                        trace_id,
                    )),
                    rag_comparison: self.build_rag_comparison_context(
                        user_message,
                        governed.as_ref(),
                        &settings,
                    ),
                    error: None,
                })
            }
            Err(e) => {
                let mut trace_id = None;
                if let Some(governed) = governed.as_ref() {
                    let session_id = self.get_or_create_rag_session_id()?;
                    let current_trace_id = format!("trace_{}", Uuid::new_v4().simple());
                    let mut trace = trace_engine::build_trace(
                        &current_trace_id,
                        Some(&session_id),
                        user_message,
                        &governed.analysis,
                        &governed.retrieval,
                        &governed.confidence,
                        &governed.decision,
                        started_at.elapsed().as_millis() as i64,
                    );
                    trace_engine::append_error(
                        &mut trace,
                        "stream_function_calling",
                        "LLM_RESPONSE_ERROR",
                        &e,
                    );
                    let _ = trace_engine::persist_trace(&trace, &[]);
                    trace_id = Some(current_trace_id);
                }
                Ok(ChatResponse {
                    text: String::new(),
                    tools_used: vec![],
                    model: settings.selected_model.clone(),
                    rag_context: Some(self.build_rag_ui_context(
                        governed.as_ref(),
                        &settings,
                        trace_id,
                    )),
                    rag_comparison: self.build_rag_comparison_context(
                        user_message,
                        governed.as_ref(),
                        &settings,
                    ),
                    error: Some(e),
                })
            }
        }
    }

    /// Limpia el historial de conversaciÃƒÂ³n
    pub fn clear_history(&self) -> Result<(), String> {
        let mut history = self.history.lock().map_err(|e| e.to_string())?;
        history.retain(|m| m.role == MessageRole::System);
        Ok(())
    }

    /// Cambia el modelo seleccionado
    pub fn set_model(&self, model_id: &str) -> Result<(), String> {
        let mut settings = self.settings.lock().map_err(|e| e.to_string())?;
        if settings.models.iter().any(|m| m.id == model_id) {
            settings.selected_model = model_id.to_string();
            settings.save()?;
            Ok(())
        } else {
            Err(format!("Model '{}' not found", model_id))
        }
    }

    /// Establece API key para un modelo
    pub fn set_api_key(&self, model_id: &str, api_key: &str) -> Result<(), String> {
        let mut settings = self.settings.lock().map_err(|e| e.to_string())?;
        settings.set_api_key(model_id, api_key)
    }

    /// Obtiene la configuraciÃƒÂ³n actual
    pub fn get_settings(&self) -> Result<AppSettings, String> {
        let settings = self.settings.lock().map_err(|e| e.to_string())?;
        Ok(settings.clone())
    }

    /// Actualiza la configuraciÃƒÂ³n
    pub fn update_settings(&self, new_settings: AppSettings) -> Result<(), String> {
        let mut settings = self.settings.lock().map_err(|e| e.to_string())?;
        *settings = new_settings;
        settings.save()
    }

    /// Detecta si Ollama y/o Docker Model Runner estÃƒÂ¡n disponibles (local o en red)
    pub async fn detect_ollama(&self) -> Vec<OllamaInstance> {
        let settings = {
            let s = self.settings.lock().unwrap_or_else(|e| e.into_inner());
            s.clone()
        };

        let mut instances = Vec::new();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(3))
            .build()
            .unwrap_or_default();

        // Ã¢â€â‚¬Ã¢â€â‚¬ Docker Model Runner (puerto 12434) Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬
        // Busca el modelo local desde el Docker Model Runner
        if let Some(dmr_model) = settings
            .models
            .iter()
            .find(|m| m.provider == "docker-model-runner")
        {
            let dmr_url = format!("{}/models", dmr_model.base_url);
            if let Ok(resp) = client.get(&dmr_url).send().await {
                if resp.status().is_success() {
                    // Docker Model Runner responde Ã¢â‚¬â€ crear instancia con modelo conocido
                    instances.push(OllamaInstance {
                        host: dmr_model
                            .base_url
                            .replace("http://", "")
                            .replace("/engines/llama.cpp/v1", "")
                            .to_string(),
                        models: vec![OllamaModel {
                            name: dmr_model.model_name.clone(),
                            size: None,
                            modified_at: None,
                        }],
                        is_local: true,
                    });
                    log::info!("Docker Model Runner detected at {}", dmr_model.base_url);
                }
            }
        }

        // Ã¢â€â‚¬Ã¢â€â‚¬ Ollama clÃƒÂ¡sico (puerto 11434) Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬
        let local_url = format!(
            "http://{}:{}/api/tags",
            settings.ollama.host, settings.ollama.port
        );
        if let Ok(resp) = client.get(&local_url).send().await {
            if let Ok(tags) = resp.json::<OllamaTagsResponse>().await {
                instances.push(OllamaInstance {
                    host: format!("{}:{}", settings.ollama.host, settings.ollama.port),
                    models: tags.models.unwrap_or_default(),
                    is_local: true,
                });
            }
        }

        // Ã¢â€â‚¬Ã¢â€â‚¬ Hosts de red Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬
        for host in &settings.ollama.network_hosts {
            let url = format!("http://{}/api/tags", host);
            if let Ok(resp) = client.get(&url).send().await {
                if let Ok(tags) = resp.json::<OllamaTagsResponse>().await {
                    instances.push(OllamaInstance {
                        host: host.clone(),
                        models: tags.models.unwrap_or_default(),
                        is_local: false,
                    });
                }
            }
        }

        instances
    }
}

#[cfg(test)]
mod tests {
    use super::{
        detect_fast_text_response, detect_local_shortcut, resolve_local_first_route, GovernedContext,
    };
    use crate::ai::live_state_retriever::LiveStateContext;
    use crate::rag::models::{
        ConfidenceAssessment, ConfidenceLevel, DecisionEnvelope, DecisionMode, DomainSpecialty,
        QueryAnalysis, QueryCategory, RiskLevel,
    };
    use crate::rag::retrieval::RetrievalBundle;

    fn governed(
        analysis: QueryAnalysis,
        decision_mode: DecisionMode,
        confidence_level: ConfidenceLevel,
        command_hits: Vec<crate::rag::models::RetrievalHit>,
    ) -> GovernedContext {
        GovernedContext {
            analysis,
            retrieval: RetrievalBundle {
                command_hits,
                ..Default::default()
            },
            confidence: ConfidenceAssessment {
                level: confidence_level.clone(),
                score: match confidence_level {
                    ConfidenceLevel::High => 0.9,
                    ConfidenceLevel::Medium => 0.7,
                    ConfidenceLevel::Low => 0.4,
                },
                reason_codes: vec!["TEST".to_string()],
                should_use_context: true,
                should_ask_clarifying_question: false,
            },
            decision: DecisionEnvelope {
                query_category: QueryCategory::ActionRequest,
                specialty: DomainSpecialty::System,
                confidence_level,
                confidence_score: 0.9,
                risk_level: RiskLevel::R0,
                decision_mode,
                requires_clarification: false,
                requires_live_state: false,
                requires_snapshot: false,
                requires_human: false,
                allowed_tools: vec![],
                denied_tools: vec![],
                reason_codes: vec![],
            },
            live_state: LiveStateContext::default(),
            prompt_context: String::new(),
        }
    }

    fn command_hit(title: &str) -> crate::rag::models::RetrievalHit {
        crate::rag::models::RetrievalHit {
            source_type: "command_or_tool".to_string(),
            source_id: "id".to_string(),
            title: title.to_string(),
            score_lexical: 0.9,
            score_vector: 0.0,
            score_final: 0.9,
            specialty: DomainSpecialty::System,
            entity_key: None,
            content: title.to_string(),
        }
    }

    #[test]
    fn answers_simple_ok_without_llm() {
        assert_eq!(detect_fast_text_response("responde solo ok"), Some("ok"));
        assert_eq!(
            detect_fast_text_response("Responde solamente OK."),
            Some("ok")
        );
    }

    #[test]
    fn detects_storage_question_as_local_tool() {
        let q = "¿Cuántas unidades de almacenamiento tiene el equipo?";
        let shortcut = detect_local_shortcut(q).expect("storage shortcut should be detected");
        assert_eq!(shortcut.0, "get_storage_summary");
    }

    #[test]
    fn detects_health_question_as_local_tool() {
        let q = "Cual es el health completo del equipo";
        let shortcut = detect_local_shortcut(q).expect("health shortcut should be detected");
        assert_eq!(shortcut.0, "health_summary");
    }

    #[test]
    fn detects_state_question_as_system_info_tool() {
        let q = "Cual es el estado actual del equipo";
        let shortcut = detect_local_shortcut(q).expect("state shortcut should be detected");
        assert_eq!(shortcut.0, "get_system_info");
    }

    #[test]
    fn detects_network_adapter_question() {
        let q = "ejecuta y dime cuantas tarjetas de red tiene el equipo y cuales son sus configuraciones";
        let shortcut = detect_local_shortcut(q).expect("shortcut should be detected");
        assert_eq!(shortcut.0, "get_network_adapters");
    }

    #[test]
    fn detects_top_process_question() {
        let q = "ejecuta y muestrame los procesos que mas recursos consumen";
        let shortcut = detect_local_shortcut(q).expect("shortcut should be detected");
        assert_eq!(shortcut.0, "list_processes");
    }

    #[test]
    fn does_not_shortcut_without_explicit_intent() {
        let q = "Muestrame los procesos que mas recursos consumen";
        let shortcut = detect_local_shortcut(q);
        assert!(shortcut.is_none());
    }

    #[test]
    fn routes_network_questions_to_local_diagnostic() {
        let analysis = QueryAnalysis {
            normalized_text: "se me cae la internet".to_string(),
            query_category: QueryCategory::ActionRequest,
            specialty: DomainSpecialty::Network,
            urgency: "high".to_string(),
            symptoms: vec!["network_down".to_string()],
            entities: vec!["internet".to_string()],
            ambiguity_score: 0.15,
            requires_clarification: false,
        };
        let governed = governed(
            analysis,
            DecisionMode::Execute,
            ConfidenceLevel::High,
            vec![command_hit("run_network_diagnostic")],
        );

        let route = resolve_local_first_route("se me cae la internet", &governed)
            .expect("network route should exist");
        assert_eq!(route.tool_name, "run_network_diagnostic");
    }

    #[test]
    fn routes_process_queries_to_process_listing() {
        let analysis = QueryAnalysis {
            normalized_text: "muestrame los procesos que mas recursos consumen".to_string(),
            query_category: QueryCategory::ActionRequest,
            specialty: DomainSpecialty::Processes,
            urgency: "normal".to_string(),
            symptoms: vec!["high_cpu".to_string()],
            entities: vec!["process".to_string()],
            ambiguity_score: 0.12,
            requires_clarification: false,
        };
        let governed = governed(
            analysis,
            DecisionMode::Execute,
            ConfidenceLevel::High,
            vec![command_hit("list_processes")],
        );

        let route = resolve_local_first_route("muestrame los procesos que mas recursos consumen", &governed)
            .expect("process route should exist");
        assert_eq!(route.tool_name, "list_processes");
    }

    #[test]
    fn routes_windows_update_questions_to_status_tool() {
        let analysis = QueryAnalysis {
            normalized_text: "lista de actualizaciones de windows".to_string(),
            query_category: QueryCategory::ActionRequest,
            specialty: DomainSpecialty::Software,
            urgency: "normal".to_string(),
            symptoms: vec!["update_failure".to_string()],
            entities: vec!["windows_update".to_string()],
            ambiguity_score: 0.10,
            requires_clarification: false,
        };
        let governed = governed(
            analysis,
            DecisionMode::Execute,
            ConfidenceLevel::High,
            vec![command_hit("get_windows_updates_status")],
        );

        let route = resolve_local_first_route("lista de actualizaciones de windows", &governed)
            .expect("update route should exist");
        assert_eq!(route.tool_name, "get_windows_updates_status");
    }

    #[test]
    fn routes_app_update_questions_to_winget_check() {
        let analysis = QueryAnalysis {
            normalized_text: "muestra las actualizaciones de aplicaciones".to_string(),
            query_category: QueryCategory::ActionRequest,
            specialty: DomainSpecialty::Software,
            urgency: "normal".to_string(),
            symptoms: Vec::new(),
            entities: vec!["windows_update".to_string()],
            ambiguity_score: 0.12,
            requires_clarification: false,
        };
        let governed = governed(
            analysis,
            DecisionMode::Execute,
            ConfidenceLevel::High,
            vec![command_hit("check_app_updates")],
        );

        let route = resolve_local_first_route("muestra las actualizaciones de aplicaciones", &governed)
            .expect("winget update route should exist");
        assert_eq!(route.tool_name, "check_app_updates");
    }

    #[test]
    fn live_core_answers_use_read_only_tools() {
        let state = crate::tools::sysinfo_tool::get_system_info();
        assert!(state.success);
        let state_answer = format!("Estado actual del equipo:\n\n{}", state.output);
        assert!(state_answer.contains("System Information"));
        assert!(!state_answer.to_lowercase().contains("no tengo acceso"));
        println!("[get_system_info] {}", state_answer.replace('\n', " | "));

        let storage = crate::tools::sysinfo_tool::get_storage_summary();
        assert!(storage.success);
        assert!(storage.output.contains("El equipo tiene"));
        assert!(!storage.output.to_lowercase().contains("no tengo acceso"));
        println!(
            "[get_storage_summary] {}",
            storage.output.replace('\n', " | ")
        );

        let health = crate::tools::phase2::health_summary();
        assert!(health.success);
        assert!(health.output.contains("Health del equipo"));
        assert!(!health.output.to_lowercase().contains("no tengo acceso"));
        println!("[health_summary] {}", health.output.replace('\n', " | "));
    }
}

/// Respuesta del chat hacia el frontend
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatResponse {
    pub text: String,
    pub tools_used: Vec<ToolUseInfo>,
    pub model: String,
    pub rag_context: Option<RagUiContext>,
    pub rag_comparison: Option<RagComparisonContext>,
    pub error: Option<String>,
}

/// InformaciÃƒÂ³n de un tool utilizado durante la respuesta
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolUseInfo {
    pub name: String,
    pub arguments: String,
}

/// Instancia de Ollama detectada
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OllamaInstance {
    pub host: String,
    pub models: Vec<OllamaModel>,
    pub is_local: bool,
}
