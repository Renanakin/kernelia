pub mod audit;
pub mod catalog_tools;
pub mod cleanup;
pub mod cloud;
pub mod drivers;
pub mod file_ops;
pub mod filesystem;
pub mod network_diagnostic;
pub mod phase10;
pub mod phase2;
pub mod phase3;
pub mod phase4;
pub mod phase5;
pub mod phase6;
pub mod phase7;
pub mod phase8;
pub mod phase9;
pub mod processes;
pub mod rbac;
pub mod registry;
pub mod report_generator;
pub mod scheduler;
pub mod security;
pub mod sysinfo_tool;
pub mod terminal;
pub mod powershell;
pub mod windows_services;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_name: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

pub struct ToolEngine;

impl ToolEngine {
    fn role_meets_minimum(role: rbac::UserRole, min_role: rbac::UserRole) -> bool {
        match (role, min_role) {
            (rbac::UserRole::Owner, _) => true,
            (rbac::UserRole::PowerUser, rbac::UserRole::PowerUser | rbac::UserRole::Viewer) => true,
            (rbac::UserRole::Viewer, rbac::UserRole::Viewer) => true,
            _ => false,
        }
    }

    pub fn get_tool_definitions() -> Vec<ToolDefinition> {
        let mut definitions = vec![
            ToolDefinition {
                name: "secure_terminal".into(),
                description: "Ejecuta un comando en la terminal del sistema con validacion de seguridad.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "command": { "type": "string", "description": "Comando a ejecutar" },
                        "working_dir": { "type": "string", "description": "Directorio de trabajo opcional" }
                    },
                    "required": ["command"]
                }),
            },
            ToolDefinition {
                name: "read_file".into(),
                description: "Lee el contenido de un archivo de texto.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Ruta absoluta o relativa del archivo a leer" }
                    },
                    "required": ["path"]
                }),
            },
            ToolDefinition {
                name: "write_file".into(),
                description: "Crea o sobrescribe un archivo con el contenido proporcionado.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Ruta del archivo" },
                        "content": { "type": "string", "description": "Contenido a escribir" }
                    },
                    "required": ["path", "content"]
                }),
            },
            ToolDefinition {
                name: "list_directory".into(),
                description: "Lista archivos y carpetas. Acepta alias como desktop, escritorio y ~.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Directorio a listar" }
                    },
                    "required": ["path"]
                }),
            },
            ToolDefinition {
                name: "get_system_info".into(),
                description: "Obtiene informacion del sistema: CPU, RAM, discos, sistema operativo y GPU.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "get_storage_summary".into(),
                description: "Resume las unidades de almacenamiento, capacidad, espacio libre y alertas de uso.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "list_processes".into(),
                description: "Lista procesos activos con uso de CPU y memoria.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "sort_by": { "type": "string", "enum": ["cpu", "memory", "name"] },
                        "limit": { "type": "integer" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "kill_process".into(),
                description: "Termina un proceso por PID o nombre.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "pid": { "type": "integer" },
                        "name": { "type": "string" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "run_network_diagnostic".into(),
                description: "Ejecuta un diagnostico completo de red.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "get_public_ip".into(),
                description: "Obtiene la direccion IP publica actual del equipo usando proveedores externos.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "list_running_services".into(),
                description: "Lista servicios de Windows en ejecucion.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "restart_service".into(),
                description: "Reinicia un servicio de Windows por nombre.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": { "name": { "type": "string" } },
                    "required": ["name"]
                }),
            },
            ToolDefinition {
                name: "generate_support_report".into(),
                description: "Genera un reporte tecnico completo.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "health_overview".into(),
                description: "Calcula health score, riesgo operacional, tendencias y anomalias.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "health_summary".into(),
                description: "Entrega resumen ejecutivo del health del equipo con score, riesgo y accion sugerida.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "run_automation_cycle".into(),
                description: "Ejecuta reglas de automatizacion inteligente y genera tickets si hay riesgo alto.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "execute_actions": { "type": "boolean", "description": "Si true ejecuta acciones, si false solo planifica" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "run_operational_suite".into(),
                description: "Ejecuta ciclo operacional completo (Windows, red, hardware, seguridad y mantenimiento) y consolida riesgos.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "execute_maintenance": { "type": "boolean", "description": "Si true ejecuta acciones de mantenimiento, si false solo diagnÃ³stico" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "scan_asset_inventory".into(),
                description: "Genera inventario automatico de hardware/software del equipo.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "create_incident_ticket".into(),
                description: "Crea ticket automatico/manual de incidente TI.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "title": { "type": "string" },
                        "category": { "type": "string" },
                        "severity": { "type": "string" },
                        "details": { "type": "string" },
                        "source": { "type": "string" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "list_incident_tickets".into(),
                description: "Lista tickets de incidente generados por el sistema.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "generate_operational_documentation".into(),
                description: "Genera documentacion automatica de acciones, riesgos y estado operacional.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "list_driver_issues".into(),
                description: "Lista controladores/dispositivos con problemas y su codigo de error.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "update_problem_drivers".into(),
                description: "Intenta actualizar/re-detectar controladores con problemas.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "search_missing_driver".into(),
                description: "Abre Windows Update en controladores opcionales para buscar driver faltante.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "analyze_junk".into(),
                description: "Escanea archivos temporales y cache para estimar espacio recuperable.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "run_cleanup".into(),
                description: "Ejecuta limpieza de archivos temporales y basura.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "target_areas": {
                            "type": "array",
                            "items": { "type": "string" }
                        }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "analyze_registry".into(),
                description: "Analiza entradas de inicio del registro de Windows.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "fix_registry".into(),
                description: "Repara problemas del registro detectados.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "schedule_maintenance".into(),
                description: "Programa una tarea de mantenimiento recurrente.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "interval_hours": { "type": "integer" },
                        "command": { "type": "string" }
                    },
                    "required": ["name", "interval_hours", "command"]
                }),
            },
            ToolDefinition {
                name: "list_scheduled_tasks".into(),
                description: "Lista tareas de mantenimiento programadas.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "delete_scheduled_task".into(),
                description: "Elimina una tarea programada por ID.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": { "id": { "type": "string" } },
                    "required": ["id"]
                }),
            },
            ToolDefinition {
                name: "toggle_scheduled_task".into(),
                description: "Activa o desactiva una tarea programada.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": { "type": "string" },
                        "enabled": { "type": "boolean" }
                    },
                    "required": ["id", "enabled"]
                }),
            },
            ToolDefinition {
                name: "upload_cloud_report".into(),
                description: "Sincroniza un diagnostico con Hackteck Cloud.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "list_cloud_reports".into(),
                description: "Obtiene historial de reportes cloud.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "register_tenant_endpoint".into(),
                description: "Registra o actualiza un endpoint dentro de un tenant para operaciÃ³n multiempresa.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "tenant_id": { "type": "string" },
                        "tenant_name": { "type": "string" },
                        "site": { "type": "string" },
                        "endpoint_id": { "type": "string" },
                        "hostname": { "type": "string" },
                        "os": { "type": "string" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "cloud_multi_tenant_overview".into(),
                description: "Consolida visiÃ³n operativa multiempresa con tenants, endpoints y salud actual.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "run_multiagent_diagnosis".into(),
                description: "Ejecuta diagnÃ³stico coordinado por agentes (red, windows, seguridad, rendimiento, helpdesk).".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "create_ticket_on_critical": { "type": "boolean" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "start_remote_support_session".into(),
                description: "Abre sesiÃ³n de soporte remoto con evidencia operativa y URI de conexiÃ³n.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "tenant_id": { "type": "string" },
                        "endpoint_id": { "type": "string" },
                        "operator": { "type": "string" },
                        "reason": { "type": "string" },
                        "transport": { "type": "string", "enum": ["rustdesk", "webrtc"] }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "close_remote_support_session".into(),
                description: "Cierra sesiÃ³n remota activa y deja trazabilidad del resultado.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "session_id": { "type": "string" },
                        "outcome": { "type": "string" }
                    },
                    "required": ["session_id"]
                }),
            },
            ToolDefinition {
                name: "list_remote_support_sessions".into(),
                description: "Lista sesiones remotas con estado, operador y evidencia.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "create_rollback_snapshot".into(),
                description: "Genera snapshot operativo previo a cambios sensibles para rollback robusto.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "reason": { "type": "string" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "rollback_to_snapshot".into(),
                description: "Restaura archivos operativos desde un snapshot registrado.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "snapshot_id": { "type": "string" }
                    },
                    "required": ["snapshot_id"]
                }),
            },
            ToolDefinition {
                name: "list_rollback_snapshots".into(),
                description: "Lista snapshots de rollback disponibles.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "attest_release_artifact".into(),
                description: "Genera attestation de integridad para un binario/artefacto de release.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_path": { "type": "string" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "verify_release_attestation".into(),
                description: "Verifica integridad del artefacto contra la Ãºltima attestation registrada.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_path": { "type": "string" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "run_phase3_smoke".into(),
                description: "Ejecuta smoke test E2E de Fase 3 (multiempresa, multiagente, remoto, rollback, trusted exec).".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "run_proactive_maintenance".into(),
                description: "Ejecuta ciclo proactivo de mantenimiento con diagnÃ³stico y autofix opcional.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "execute_actions": { "type": "boolean" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "list_proactive_alerts".into(),
                description: "Lista alertas proactivas histÃ³ricas generadas por el motor de mantenimiento.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": { "type": "integer" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "schedule_proactive_automation".into(),
                description: "Programa tareas automÃ¡ticas de mantenimiento preventivo y smoke de validaciÃ³n.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "interval_hours": { "type": "integer" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "recommend_model_route".into(),
                description: "Recomienda modelo IA segÃºn tipo de tarea, privacidad y urgencia.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "task_kind": { "type": "string", "enum": ["general", "tool_heavy", "long_context", "security"] },
                        "privacy_level": { "type": "string", "enum": ["strict", "balanced", "open"] },
                        "urgency": { "type": "string", "enum": ["low", "normal", "high"] }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "apply_recommended_model_route".into(),
                description: "Aplica y persiste selecciÃ³n de modelo recomendada para el siguiente ciclo de chat.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "task_kind": { "type": "string", "enum": ["general", "tool_heavy", "long_context", "security"] },
                        "privacy_level": { "type": "string", "enum": ["strict", "balanced", "open"] },
                        "urgency": { "type": "string", "enum": ["low", "normal", "high"] }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "run_phase4_smoke".into(),
                description: "Ejecuta smoke test E2E de Fase 4 (proactividad, scheduler y multimodelo).".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "create_support_case".into(),
                description: "Crea caso de soporte enterprise asociado a ticket cloud.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "ticket_id": { "type": "string" },
                        "severity": { "type": "string" },
                        "customer": { "type": "string" },
                        "summary": { "type": "string" },
                        "assigned_team": { "type": "string" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "list_support_cases".into(),
                description: "Lista casos de soporte enterprise.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": { "type": "integer" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "get_enterprise_dashboard".into(),
                description: "Consolida KPIs enterprise: cloud, tickets, alertas y casos.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "generate_advanced_reporting".into(),
                description: "Genera reporte ejecutivo avanzado para operaciÃ³n enterprise.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "run_phase5_smoke".into(),
                description: "Ejecuta smoke test E2E de Fase 5 (cloud sync, casos y dashboard).".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "run_kernel_slowpc_diagnostic".into(),
                description: "Ejecuta diagnÃ³stico KernelIA para equipos lentos con causas y acciones.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "run_kernel_network_playbook".into(),
                description: "Ejecuta playbook de diagnÃ³stico de red orientado a soporte KernelIA.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "validate_kernel_guardrails".into(),
                description: "Valida que comandos destructivos sean bloqueados por guardrails de seguridad.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "command": { "type": "string" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "run_kernel_autonomous_workflow".into(),
                description: "Ejecuta workflow autonomo IF/THEN con simulacion o ejecucion y verificacion posterior.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "execute_actions": { "type": "boolean" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "list_kernel_diagnostics".into(),
                description: "Lista historial de diagnÃ³sticos KernelIA almacenados.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": { "type": "integer" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "list_kernel_automation_runs".into(),
                description: "Lista historico de workflows autonomos de Fase 6.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": { "type": "integer" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "generate_kernelia_readiness_report".into(),
                description: "Genera reporte de readiness de diagnÃ³sticos y guardrails KernelIA.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "run_phase6_smoke".into(),
                description: "Ejecuta smoke test E2E de Fase 6 para diagnÃ³sticos y seguridad KernelIA.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "run_latency_probe".into(),
                description: "Mide latencia base de operaciones nÃºcleo y calcula mÃ©tricas de rendimiento.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "iterations": { "type": "integer" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "run_tool_benchmark".into(),
                description: "Ejecuta benchmark de una herramienta crÃ­tica y reporta avg/p95.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "tool": { "type": "string", "enum": ["get_system_info", "list_processes", "run_network_diagnostic"] },
                        "iterations": { "type": "integer" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "get_performance_kpis".into(),
                description: "Consolida KPIs de rendimiento y latencia histÃ³rica.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": { "type": "integer" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "generate_performance_report".into(),
                description: "Genera reporte de performance con KPIs y muestras recientes.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "get_noc_global_status".into(),
                description: "Consolida estado NOC global multiempresa con SLA base y riesgo operativo.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "register_saas_license".into(),
                description: "Registra o actualiza licencia SaaS por tenant (plan, seats, estado).".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "tenant_id": { "type": "string" },
                        "plan": { "type": "string", "enum": ["basic", "business", "enterprise"] },
                        "seats": { "type": "integer" },
                        "status": { "type": "string" }
                    },
                    "required": ["tenant_id"]
                }),
            },
            ToolDefinition {
                name: "list_saas_licenses".into(),
                description: "Lista licencias SaaS registradas por tenant.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "generate_enterprise_noc_report".into(),
                description: "Genera reporte enterprise NOC consolidado con estado global y licencias SaaS.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "run_phase7_smoke".into(),
                description: "Ejecuta smoke test E2E de Fase 7 para rendimiento y latencia.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "detect_performance_anomalies".into(),
                description: "Detecta anomalÃ­as de fiabilidad usando baseline de p95 y tasa de Ã©xito histÃ³rica.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": { "type": "integer" },
                        "p95_multiplier": { "type": "number" },
                        "min_success_rate": { "type": "number" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "calculate_sla_status".into(),
                description: "Calcula estado de SLA segÃºn tasa de Ã©xito objetivo.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": { "type": "integer" },
                        "target_success_rate": { "type": "number" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "recommend_reliability_actions".into(),
                description: "Genera acciones recomendadas de resiliencia en base a anomalÃ­as recientes.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": { "type": "integer" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "generate_reliability_report".into(),
                description: "Genera reporte de fiabilidad con SLA, anomalÃ­as y acciones sugeridas.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "predict_operational_incidents".into(),
                description: "Predice incidentes operacionales futuros usando series temporales y anomalias recientes.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": { "type": "integer" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "explain_root_cause".into(),
                description: "Explica causa raiz probable de degradaciones con nivel de confianza.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": { "type": "integer" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "generate_autonomous_playbook".into(),
                description: "Genera playbook autonomo de prevencion y respuesta para incidentes operacionales.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": { "type": "integer" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "run_phase8_smoke".into(),
                description: "Ejecuta smoke test E2E de Fase 8 para fiabilidad, SLA y resiliencia.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "assess_self_healing_readiness".into(),
                description: "Evalua readiness de autocuracion a partir de riesgo y estado SLA.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": { "type": "integer" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "generate_self_healing_plan".into(),
                description: "Genera plan preventivo de autocuracion segun readiness operacional.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": { "type": "integer" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "execute_self_healing_cycle".into(),
                description: "Ejecuta o simula un ciclo de autocuracion con mitigaciones base.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "execute_actions": { "type": "boolean" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "list_self_healing_runs".into(),
                description: "Lista historico de ciclos de autocuracion ejecutados/simulados.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": { "type": "integer" }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "run_phase9_smoke".into(),
                description: "Ejecuta smoke test E2E de Fase 9 para autocuracion y prevencion.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "assess_go_live_readiness".into(),
                description: "Evalua readiness de salida a produccion con score consolidado.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "verify_go_live_controls".into(),
                description: "Verifica controles de gobernanza y seguridad para go-live.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "generate_go_live_bundle".into(),
                description: "Genera paquete de evidencia y scorecard ejecutiva de go-live.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
            ToolDefinition {
                name: "run_phase10_smoke".into(),
                description: "Ejecuta smoke test E2E de Fase 10 para cierre go-live.".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
        ];

        definitions.extend(catalog_tools::tool_definitions());
        definitions
    }

    pub async fn execute(
        app: &AppHandle,
        tool_name: &str,
        args: &serde_json::Value,
        role: rbac::UserRole,
    ) -> ToolResult {
        // Validar existencia de la tool antes de RBAC para evitar falsos "acceso denegado"
        let policy = match catalog_tools::get_tool_policy(tool_name) {
            Some(p) => p,
            None => {
                let res = ToolResult {
                    tool_name: tool_name.to_string(),
                    success: false,
                    output: String::new(),
                    error: Some(format!("Tool '{}' not found", tool_name)),
                };
                audit::log_action(app, tool_name, args, false, res.error.clone());
                return res;
            }
        };

        if !Self::role_meets_minimum(role, policy.min_role) {
            let res = ToolResult {
                tool_name: tool_name.to_string(),
                success: false,
                output: String::new(),
                error: Some(format!(
                    "ACCESO DENEGADO: El rol '{:?}' no cumple mÃ­nimo '{:?}' para '{}'.",
                    role, policy.min_role, tool_name
                )),
            };
            audit::log_action(app, tool_name, args, false, res.error.clone());
            return res;
        }

        // ValidaciÃ³n RBAC
        if let Err(err) = rbac::ensure_permission(role, tool_name) {
            let res = ToolResult {
                tool_name: tool_name.to_string(),
                success: false,
                output: String::new(),
                error: Some(err.clone()),
            };
            audit::log_action(app, tool_name, args, false, Some(err));
            return res;
        }

        let result = match tool_name {
            "secure_terminal" | "run_command" => {
                let command = args["command"].as_str().unwrap_or("");
                let working_dir = args["working_dir"].as_str();
                terminal::run_command(command, working_dir).await
            }
            "read_file" => {
                let path = args["path"].as_str().unwrap_or("");
                filesystem::read_file(path)
            }
            "write_file" => {
                let path = args["path"].as_str().unwrap_or("");
                let content = args["content"].as_str().unwrap_or("");
                filesystem::write_file(path, content)
            }
            "list_directory" => {
                let path = args["path"].as_str().unwrap_or("desktop");
                filesystem::list_directory(path)
            }
            "get_system_info" => sysinfo_tool::get_system_info(),
            "get_storage_summary" => sysinfo_tool::get_storage_summary(),
            "list_processes" => {
                let sort_by = args["sort_by"].as_str().unwrap_or("memory");
                let limit = args["limit"].as_u64().unwrap_or(20) as usize;
                processes::list_processes(sort_by, limit)
            }
            "kill_process" => {
                let pid = args["pid"].as_u64().map(|p| p as u32);
                let name = args["name"].as_str();
                processes::kill_process(pid, name)
            }
            "run_network_diagnostic" => network_diagnostic::run_network_diagnostic(),
            "get_public_ip" => network_diagnostic::get_public_ip().await,
            "list_running_services" => windows_services::list_running_services(),
            "restart_service" => {
                let name = args["name"].as_str().unwrap_or("");
                windows_services::restart_service(name)
            }
            "generate_support_report" => report_generator::generate_support_report(),
            "health_overview" => phase2::health_overview(),
            "health_summary" => phase2::health_summary(),
            "scan_asset_inventory" => phase2::scan_asset_inventory(),
            "create_incident_ticket" => phase2::create_incident_ticket(args),
            "list_incident_tickets" => phase2::list_incident_tickets(),
            "generate_operational_documentation" => phase2::generate_operational_documentation(),
            "run_automation_cycle" => {
                let execute_actions = args
                    .get("execute_actions")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                phase2::run_automation_cycle(app, role, execute_actions).await
            }
            "run_operational_suite" => {
                let execute_maintenance = args
                    .get("execute_maintenance")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                phase2::run_operational_suite(app, role, execute_maintenance).await
            }
            "list_driver_issues" => drivers::list_driver_issues(),
            "update_problem_drivers" => drivers::update_problem_drivers(),
            "search_missing_driver" => drivers::search_missing_driver(),
            "analyze_junk" => cleanup::analyze_junk(),
            "run_cleanup" => {
                let areas = args["target_areas"].as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                });
                cleanup::run_cleanup(areas).await
            }
            "analyze_registry" => registry::analyze_registry().await,
            "fix_registry" => registry::fix_registry().await,
            "schedule_maintenance" => {
                let name = args["name"].as_str().unwrap_or("Mantenimiento");
                let interval = args["interval_hours"].as_u64().unwrap_or(168);
                let command = args["command"].as_str().unwrap_or("run_cleanup");
                scheduler::schedule_maintenance(name, interval, command)
            }
            "list_scheduled_tasks" => scheduler::list_scheduled_tasks(),
            "delete_scheduled_task" => {
                let id = args["id"].as_str().unwrap_or("");
                scheduler::delete_scheduled_task(id)
            }
            "toggle_scheduled_task" => {
                let id = args["id"].as_str().unwrap_or("");
                let enabled = args["enabled"].as_bool().unwrap_or(true);
                scheduler::toggle_scheduled_task(id, enabled)
            }
            "upload_cloud_report" => cloud::upload_report(app),
            "list_cloud_reports" => cloud::list_cloud_reports(),
            "register_tenant_endpoint" => phase3::register_tenant_endpoint(args),
            "cloud_multi_tenant_overview" => phase3::cloud_multi_tenant_overview(),
            "run_multiagent_diagnosis" => phase3::run_multiagent_diagnosis(args),
            "start_remote_support_session" => phase3::start_remote_support_session(args),
            "close_remote_support_session" => phase3::close_remote_support_session(args),
            "list_remote_support_sessions" => phase3::list_remote_support_sessions(),
            "create_rollback_snapshot" => phase3::create_rollback_snapshot(args),
            "rollback_to_snapshot" => phase3::rollback_to_snapshot(args),
            "list_rollback_snapshots" => phase3::list_rollback_snapshots(),
            "attest_release_artifact" => phase3::attest_release_artifact(args),
            "verify_release_attestation" => phase3::verify_release_attestation(args),
            "run_phase3_smoke" => phase3::run_phase3_smoke(),
            "run_proactive_maintenance" => {
                let execute_actions = args
                    .get("execute_actions")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                phase4::run_proactive_maintenance(app, role, execute_actions).await
            }
            "list_proactive_alerts" => phase4::list_proactive_alerts(args),
            "schedule_proactive_automation" => phase4::schedule_proactive_automation(args),
            "recommend_model_route" => phase4::recommend_model_route(args),
            "apply_recommended_model_route" => phase4::apply_recommended_model_route(args),
            "run_phase4_smoke" => phase4::run_phase4_smoke(app, role).await,
            "create_support_case" => phase5::create_support_case(args),
            "list_support_cases" => phase5::list_support_cases(args),
            "get_enterprise_dashboard" => phase5::get_enterprise_dashboard(),
            "generate_advanced_reporting" => phase5::generate_advanced_reporting(),
            "run_phase5_smoke" => phase5::run_phase5_smoke(app).await,
            "run_kernel_slowpc_diagnostic" => phase6::run_kernel_slowpc_diagnostic(),
            "run_kernel_network_playbook" => phase6::run_kernel_network_playbook(),
            "validate_kernel_guardrails" => phase6::validate_kernel_guardrails(args),
            "run_kernel_autonomous_workflow" => {
                let execute_actions = args
                    .get("execute_actions")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                phase6::run_kernel_autonomous_workflow(app, role, execute_actions).await
            }
            "list_kernel_diagnostics" => phase6::list_kernel_diagnostics(args),
            "list_kernel_automation_runs" => phase6::list_kernel_automation_runs(args),
            "generate_kernelia_readiness_report" => phase6::generate_kernelia_readiness_report(),
            "run_phase6_smoke" => phase6::run_phase6_smoke(),
            "run_latency_probe" => phase7::run_latency_probe(args),
            "run_tool_benchmark" => phase7::run_tool_benchmark(args),
            "get_performance_kpis" => phase7::get_performance_kpis(args),
            "generate_performance_report" => phase7::generate_performance_report(),
            "get_noc_global_status" => phase7::get_noc_global_status(),
            "register_saas_license" => phase7::register_saas_license(args),
            "list_saas_licenses" => phase7::list_saas_licenses(),
            "generate_enterprise_noc_report" => phase7::generate_enterprise_noc_report(),
            "run_phase7_smoke" => phase7::run_phase7_smoke(),
            "detect_performance_anomalies" => phase8::detect_performance_anomalies(args),
            "calculate_sla_status" => phase8::calculate_sla_status(args),
            "recommend_reliability_actions" => phase8::recommend_reliability_actions(args),
            "generate_reliability_report" => phase8::generate_reliability_report(),
            "predict_operational_incidents" => phase8::predict_operational_incidents(args),
            "explain_root_cause" => phase8::explain_root_cause(args),
            "generate_autonomous_playbook" => phase8::generate_autonomous_playbook(args),
            "run_phase8_smoke" => phase8::run_phase8_smoke(),
            "assess_self_healing_readiness" => phase9::assess_self_healing_readiness(args),
            "generate_self_healing_plan" => phase9::generate_self_healing_plan(args),
            "execute_self_healing_cycle" => phase9::execute_self_healing_cycle(args),
            "list_self_healing_runs" => phase9::list_self_healing_runs(args),
            "run_phase9_smoke" => phase9::run_phase9_smoke(),
            "assess_go_live_readiness" => phase10::assess_go_live_readiness(args),
            "verify_go_live_controls" => phase10::verify_go_live_controls(args),
            "generate_go_live_bundle" => phase10::generate_go_live_bundle(),
            "run_phase10_smoke" => phase10::run_phase10_smoke(),
            tool_name if catalog_tools::is_catalog_tool(tool_name) => {
                catalog_tools::execute_catalog_tool(tool_name, args).await
            }
            "read_file_ops" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                file_ops::read_file(path)
            }
            "write_file_ops" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let content = args
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                file_ops::write_file(path, content)
            }
            "get_audit_logs" => {
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
                let logs = audit::read_audit_logs(limit);
                ToolResult {
                    tool_name: "get_audit_logs".to_string(),
                    success: true,
                    output: serde_json::to_string(&logs).unwrap_or_default(),
                    error: None,
                }
            }
            _ => ToolResult {
                tool_name: tool_name.to_string(),
                success: false,
                output: String::new(),
                error: Some(format!("Tool '{}' not found", tool_name)),
            },
        };

        // Avoid recursive/noisy audit entries when the audit panel itself reads logs.
        if tool_name != "get_audit_logs" {
            audit::log_action(app, tool_name, args, result.success, result.error.clone());
        }
        result
    }
}
