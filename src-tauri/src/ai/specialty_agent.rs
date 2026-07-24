use crate::ai::live_state_retriever::LiveStateContext;
use crate::rag::models::{DecisionEnvelope, DomainSpecialty, RetrievalHit};
use crate::rag::retrieval::RetrievalBundle;

#[derive(Debug, Clone)]
pub struct SpecialtyAgentProfile {
    pub specialty: DomainSpecialty,
    pub name: &'static str,
    pub mission: &'static str,
    pub response_contract: &'static str,
    pub required_evidence: &'static [&'static str],
    pub preferred_tools: &'static [&'static str],
    pub avoid_behaviors: &'static [&'static str],
}

pub fn build_specialty_agent_profile(specialty: &DomainSpecialty) -> SpecialtyAgentProfile {
    match specialty {
        DomainSpecialty::Network => SpecialtyAgentProfile {
            specialty: specialty.clone(),
            name: "NetworkAgent",
            mission: "Diagnosticar conectividad, DNS, gateway, latencia y configuracion de red.",
            response_contract: "Devuelve: estado de red, evidencia, causa probable, accion recomendada y si requiere escalado.",
            required_evidence: &[
                "run_network_diagnostic",
                "get_network_adapters",
                "get_local_ip",
                "get_default_gateway",
                "get_dns_servers",
            ],
            preferred_tools: &[
                "run_network_diagnostic",
                "get_network_adapters",
                "get_local_ip",
                "get_default_gateway",
                "get_dns_servers",
                "dns_lookup",
                "ping_host",
                "traceroute_host",
            ],
            avoid_behaviors: &[
                "no inventar estado de internet",
                "no asumir DNS sin diagnostico",
                "no recomendar cambios destructivos",
            ],
        },
        DomainSpecialty::Processes => SpecialtyAgentProfile {
            specialty: specialty.clone(),
            name: "ProcessAgent",
            mission: "Detectar procesos que consumen CPU, RAM o bloquean el sistema.",
            response_contract: "Devuelve: top procesos, recurso dominante, riesgo y accion segura.",
            required_evidence: &["list_processes", "get_cpu_usage", "get_memory_usage"],
            preferred_tools: &[
                "list_processes",
                "get_top_processes",
                "find_high_cpu_processes",
                "find_high_memory_processes",
                "get_process_detail",
                "get_cpu_usage",
                "get_memory_usage",
            ],
            avoid_behaviors: &[
                "no diagnosticar sin lista de procesos",
                "no sugerir kill_process salvo escalado",
            ],
        },
        DomainSpecialty::Services => SpecialtyAgentProfile {
            specialty: specialty.clone(),
            name: "ServicesAgent",
            mission: "Verificar servicios de Windows, cola de impresion, arranque y estado operativo.",
            response_contract: "Devuelve: servicio afectado, estado, evidencia y accion recomendada.",
            required_evidence: &["list_running_services", "get_service_status", "restart_service"],
            preferred_tools: &[
                "list_running_services",
                "list_services",
                "get_service_status",
                "restart_service",
                "start_service",
                "stop_service",
            ],
            avoid_behaviors: &["no confundir servicio con proceso", "no reiniciar sin evidencia"],
        },
        DomainSpecialty::Performance => SpecialtyAgentProfile {
            specialty: specialty.clone(),
            name: "PerformanceAgent",
            mission: "Analizar CPU, RAM, disco, latencia y degradacion del rendimiento.",
            response_contract: "Devuelve: mediciones, cuello de botella, tendencia y accion concreta.",
            required_evidence: &["get_system_info", "get_storage_summary", "list_processes"],
            preferred_tools: &[
                "get_system_info",
                "get_storage_summary",
                "list_processes",
                "get_cpu_usage",
                "get_memory_usage",
                "get_disk_usage",
                "detect_performance_anomalies",
            ],
            avoid_behaviors: &["no generalizar sin metricas", "no usar lenguaje vago"],
        },
        DomainSpecialty::Security | DomainSpecialty::SensitiveOps => SpecialtyAgentProfile {
            specialty: specialty.clone(),
            name: "SecurityAgent",
            mission: "Evaluar riesgo, incidentes, cambios seguros y cumplimiento de guardrails.",
            response_contract: "Devuelve: riesgo, evidencia, si requiere escalado y accion segura.",
            required_evidence: &["health_summary", "verify_go_live_controls", "get_audit_logs"],
            preferred_tools: &[
                "health_summary",
                "health_overview",
                "get_audit_logs",
                "verify_go_live_controls",
                "assess_go_live_readiness",
                "validate_kernel_guardrails",
            ],
            avoid_behaviors: &[
                "no ejecutar comandos destructivos",
                "no asumir seguridad sin logs",
                "no ocultar riesgo alto",
            ],
        },
        DomainSpecialty::Filesystem => SpecialtyAgentProfile {
            specialty: specialty.clone(),
            name: "FilesystemAgent",
            mission: "Evaluar almacenamiento, rutas, permisos y estructura de archivos.",
            response_contract: "Devuelve: ruta afectada, evidencia y accion no destructiva.",
            required_evidence: &["get_system_info", "get_storage_summary", "list_directory"],
            preferred_tools: &["list_directory", "read_file", "get_system_info", "get_storage_summary"],
            avoid_behaviors: &["no borrar ni reescribir archivos sin permiso", "no adivinar rutas"],
        },
        DomainSpecialty::Drivers => SpecialtyAgentProfile {
            specialty: specialty.clone(),
            name: "DriversAgent",
            mission: "Detectar controladores con fallo, codigo 43 y estado de hardware.",
            response_contract: "Devuelve: dispositivo o driver, codigo de error, evidencia y paso siguiente.",
            required_evidence: &["list_driver_issues", "search_missing_driver"],
            preferred_tools: &["list_driver_issues", "update_problem_drivers", "search_missing_driver"],
            avoid_behaviors: &["no inferir hardware sin lectura real"],
        },
        DomainSpecialty::System | DomainSpecialty::Telemetry | DomainSpecialty::Maintenance | DomainSpecialty::Software => {
            SpecialtyAgentProfile {
                specialty: specialty.clone(),
                name: "SystemAgent",
                mission: "Consolidar estado del equipo, salud, inventario y mantenimiento seguro.",
                response_contract: "Devuelve: estado global, evidencia, riesgo y accion recomendada.",
                required_evidence: &["get_system_info", "health_summary", "scan_asset_inventory"],
                preferred_tools: &[
                    "get_system_info",
                    "get_storage_summary",
                    "health_summary",
                    "health_overview",
                    "scan_asset_inventory",
                    "generate_operational_documentation",
                ],
                avoid_behaviors: &["no responder con vaguedades", "no omitir metricas clave"],
            }
        }
        _ => SpecialtyAgentProfile {
            specialty: specialty.clone(),
            name: "GeneralAgent",
            mission: "Responder solo con evidencia cuando la especialidad no esta clara.",
            response_contract: "Devuelve: resumen breve, evidencia disponible y si hace falta clarificar.",
            required_evidence: &["get_system_info"],
            preferred_tools: &["get_system_info", "run_network_diagnostic", "health_summary"],
            avoid_behaviors: &["no inventar una especialidad", "no suponer una accion sin evidencia"],
        },
    }
}

pub fn preferred_tools_for_message(message: &str) -> Vec<&'static str> {
    let text = normalize_message(message);

    if text.contains("tarjeta")
        || text.contains("adaptador")
        || text.contains("red")
        || text.contains("ip")
        || text.contains("dns")
        || text.contains("gateway")
        || text.contains("wifi")
    {
        return vec![
            "run_network_diagnostic",
            "get_network_adapters",
            "get_local_ip",
            "get_default_gateway",
            "get_dns_servers",
            "get_wifi_info",
            "get_public_ip",
            "ping_host",
            "dns_lookup",
            "traceroute_host",
            "test_tcp_port",
            "get_network_usage",
        ];
    }

    if text.contains("estado")
        || text.contains("salud")
        || text.contains("health")
        || text.contains("disco")
        || text.contains("almacenamiento")
    {
        return vec![
            "health_summary",
            "get_system_info",
            "get_storage_summary",
            "health_overview",
            "scan_asset_inventory",
            "run_operational_suite",
        ];
    }

    if text.contains("proceso")
        || text.contains("cpu")
        || text.contains("memoria")
        || text.contains("rendimiento")
        || text.contains("lento")
    {
        return vec![
            "list_processes",
            "get_top_processes",
            "find_high_cpu_processes",
            "find_high_memory_processes",
            "get_process_detail",
            "get_cpu_usage",
            "get_memory_usage",
            "get_disk_usage",
            "get_system_info",
        ];
    }

    if text.contains("servicio") || text.contains("service") {
        return vec![
            "list_running_services",
            "list_services",
            "get_service_status",
            "restart_service",
            "start_service",
            "stop_service",
        ];
    }

    vec![
        "get_system_info",
        "list_processes",
        "run_network_diagnostic",
        "get_network_adapters",
        "get_local_ip",
        "get_dns_servers",
        "list_running_services",
        "generate_support_report",
    ]
}

pub fn render_specialty_agent_context(
    profile: &SpecialtyAgentProfile,
    decision: &DecisionEnvelope,
    retrieval: &RetrievalBundle,
    live_state: &LiveStateContext,
) -> String {
    let allowed_tools = if decision.allowed_tools.is_empty() {
        profile.preferred_tools.join(", ")
    } else {
        decision.allowed_tools.join(", ")
    };
    let primary_hits = render_hit_titles(&retrieval.knowledge_hits, 3);
    let command_hits = render_hit_titles(&retrieval.command_hits, 4);
    let live_summary = if live_state.summary.is_empty() {
        "sin estado vivo adicional".to_string()
    } else {
        live_state.summary.join(" | ")
    };

    format!(
        "[KERNEL_SPECIALIST]\nname={}\nspecialty={:?}\nmission={}\nresponse_contract={}\nrequired_evidence={}\npreferred_tools={}\navoid_behaviors={}\n\n[KERNEL_SPECIALIST_CONTEXT]\nallowed_tools={}\nknowledge_titles={}\ncommand_titles={}\nlive_state={}\n",
        profile.name,
        profile.specialty,
        profile.mission,
        profile.response_contract,
        profile.required_evidence.join(", "),
        profile.preferred_tools.join(", "),
        profile.avoid_behaviors.join(", "),
        allowed_tools,
        primary_hits,
        command_hits,
        live_summary
    )
}

fn render_hit_titles(hits: &[RetrievalHit], limit: usize) -> String {
    let titles = hits
        .iter()
        .take(limit)
        .map(|hit| hit.title.clone())
        .collect::<Vec<_>>();

    if titles.is_empty() {
        "none".to_string()
    } else {
        titles.join(" | ")
    }
}

fn normalize_message(message: &str) -> String {
    let mut out = String::with_capacity(message.len());
    for ch in message.to_lowercase().chars() {
        let normalized = match ch {
            'á' | 'à' | 'ä' | 'â' | 'ã' | 'å' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' | 'õ' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            'ç' => 'c',
            c if c.is_ascii_alphanumeric() || c.is_ascii_whitespace() => c,
            _ => ' ',
        };
        out.push(normalized);
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::live_state_retriever::LiveStateContext;
    use crate::rag::models::{
        ConfidenceLevel, DecisionEnvelope, DecisionMode, QueryCategory, RiskLevel,
    };
    use crate::rag::retrieval::RetrievalBundle;

    #[test]
    fn builds_network_agent_profile() {
        let profile = build_specialty_agent_profile(&DomainSpecialty::Network);
        assert_eq!(profile.name, "NetworkAgent");
        assert!(profile.preferred_tools.contains(&"run_network_diagnostic"));
    }

    #[test]
    fn renders_specialist_context_with_contract() {
        let profile = build_specialty_agent_profile(&DomainSpecialty::System);
        let decision = DecisionEnvelope {
            query_category: QueryCategory::Specific,
            specialty: DomainSpecialty::System,
            confidence_level: ConfidenceLevel::High,
            confidence_score: 0.92,
            risk_level: RiskLevel::R0,
            decision_mode: DecisionMode::Explain,
            requires_clarification: false,
            requires_live_state: true,
            requires_snapshot: false,
            requires_human: false,
            allowed_tools: vec!["get_system_info".to_string()],
            denied_tools: Vec::new(),
            reason_codes: vec!["TEST".to_string()],
        };
        let ctx = render_specialty_agent_context(
            &profile,
            &decision,
            &RetrievalBundle::default(),
            &LiveStateContext::default(),
        );
        assert!(ctx.contains("[KERNEL_SPECIALIST]"));
        assert!(ctx.contains("response_contract"));
        assert!(ctx.contains("get_system_info"));
    }

    #[test]
    fn preferred_tools_follow_specialty_keywords() {
        let tools = preferred_tools_for_message("ejecuta y muestrame los procesos que mas recursos consumen");
        assert!(tools.iter().any(|tool| *tool == "list_processes"));
    }
}
