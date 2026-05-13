use crate::ai::AiRouter;
use crate::commands::common;
use crate::tools;
use std::sync::{Arc, Mutex};
use tauri::State;

/// Obtiene informaciÃ³n del sistema (CPU, RAM, discos, etc.)
#[tauri::command]
pub async fn get_system_info() -> Result<String, String> {
    Ok(tools::sysinfo_tool::get_system_info_json())
}

/// Lista los procesos del sistema
#[tauri::command]
pub async fn list_processes(
    sort_by: Option<String>,
    limit: Option<usize>,
) -> Result<String, String> {
    let sort = common::normalize_sort_by(sort_by)?;
    let lim = common::normalize_limit(limit, 5, 200);
    Ok(tools::processes::list_processes_json(&sort, lim))
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn list_processes_compat(
    sort_by: Option<String>,
    sortBy: Option<String>,
    limit: Option<usize>,
) -> Result<String, String> {
    let normalized_sort = common::normalize_sort_by(sort_by.or(sortBy))?;
    let normalized_limit = common::normalize_limit(limit, 5, 200);
    Ok(tools::processes::list_processes_json(
        &normalized_sort,
        normalized_limit,
    ))
}

/// Obtiene la versiÃ³n de la aplicaciÃ³n
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Ejecuta diagnÃ³sticos de red
#[tauri::command]
pub async fn run_network_diagnostic() -> Result<String, String> {
    Ok(tools::network_diagnostic::run_network_diagnostic_json())
}

/// Lista servicios en ejecucion
#[tauri::command]
pub async fn list_running_services() -> Result<String, String> {
    let result = tools::windows_services::list_running_services();
    if result.success {
        Ok(result.output)
    } else {
        Err(result
            .error
            .unwrap_or_else(|| "Error al listar servicios".into()))
    }
}

/// Reinicia un servicio
#[tauri::command]
pub async fn restart_service(name: String) -> Result<String, String> {
    common::validate_service_name(&name)?;
    let result = tools::windows_services::restart_service(&name);
    if result.success {
        Ok(result.output)
    } else {
        Err(result
            .error
            .unwrap_or_else(|| "Error al reiniciar servicio".into()))
    }
}

/// Genera el reporte de soporte
#[tauri::command]
pub async fn generate_support_report() -> Result<String, String> {
    let result = tools::report_generator::generate_support_report();
    if result.success {
        Ok(result.output)
    } else {
        Err(result
            .error
            .unwrap_or_else(|| "Error al generar reporte".into()))
    }
}

/// Obtiene el historial de auditorÃ­a
#[tauri::command]
pub async fn get_audit_logs(
    _app: tauri::AppHandle,
) -> Result<Vec<tools::audit::AuditEntry>, String> {
    Ok(tools::audit::read_audit_logs(100))
}

/// Ejecuta una herramienta de forma genérica desde el frontend
#[tauri::command]
pub async fn execute_tool(
    app: tauri::AppHandle,
    router: State<'_, Arc<AiRouter>>,
    name: String,
    args: serde_json::Value,
) -> Result<tools::ToolResult, String> {
    common::validate_tool_name(&name)?;
    let settings = router.get_settings()?;
    if !settings.auth.is_authenticated {
        return Err("Debe iniciar sesion antes de ejecutar herramientas".to_string());
    }
    let requires_megaboss = tools::rbac::is_owner_only_tool(&name);
    if requires_megaboss && !settings.is_megaboss_unlocked() {
        return Ok(tools::ToolResult {
            tool_name: name,
            success: false,
            output: String::new(),
            error: Some("Privilegio MegaBoss requerido. Desbloquea con contraseña en Configuración para ejecutar comandos de máximo privilegio.".to_string()),
        });
    }
    Ok(tools::ToolEngine::execute(&app, &name, &args, settings.user_role).await)
}

/// Devuelve los Quick Checks permitidos para el rol activo del usuario
#[tauri::command]
pub async fn get_quick_checks(
    router: State<'_, Arc<AiRouter>>,
) -> Result<Vec<crate::config::settings::QuickCheck>, String> {
    let settings = router.get_settings()?;
    if !settings.auth.is_authenticated {
        return Err("Debe iniciar sesion antes de consultar quick checks".to_string());
    }
    let role = settings.user_role;

    let allowed: Vec<_> = settings
        .quick_checks
        .into_iter()
        .filter(|qc| {
            qc.required_permissions
                .iter()
                .all(|perm| crate::tools::rbac::ensure_permission(role, perm).is_ok())
        })
        .collect();

    Ok(allowed)
}

/// Ejecuta un Quick Check por su ID con validación RBAC completa
#[tauri::command]
pub async fn run_quick_check(
    app: tauri::AppHandle,
    router: State<'_, Arc<AiRouter>>,
    quick_check_id: String,
) -> Result<crate::config::settings::QuickCheckResult, String> {
    use crate::config::settings::{QuickCheckResult, QuickCheckType};

    common::validate_quick_check_id(&quick_check_id)?;

    let settings = router.get_settings()?;
    if !settings.auth.is_authenticated {
        return Err("Debe iniciar sesion antes de ejecutar quick checks".to_string());
    }
    let role = settings.user_role;

    let qc = settings
        .quick_checks
        .iter()
        .find(|q| q.id == quick_check_id)
        .ok_or_else(|| format!("Quick Check '{}' no encontrado.", quick_check_id))
        .cloned()?;

    // Validar permisos (PoLP) — backend es la última barrera
    for perm in &qc.required_permissions {
        crate::tools::rbac::ensure_permission(role, perm)?;
    }

    match qc.kind {
        QuickCheckType::DirectTool => {
            let mut outputs: Vec<String> = Vec::new();
            let args = serde_json::json!({});

            for tool_name in &qc.tool_pipeline {
                if crate::tools::rbac::is_owner_only_tool(tool_name)
                    && !settings.is_megaboss_unlocked()
                {
                    return Ok(QuickCheckResult {
                        id: qc.id.clone(),
                        label: qc.label.clone(),
                        success: false,
                        output: String::new(),
                        error: Some(
                            "Privilegio MegaBoss requerido para este Quick Check.".to_string(),
                        ),
                        kind: "direct_tool".into(),
                    });
                }
                let result = tools::ToolEngine::execute(&app, tool_name, &args, role).await;
                if result.success {
                    outputs.push(result.output);
                } else {
                    return Ok(QuickCheckResult {
                        id: qc.id.clone(),
                        label: qc.label.clone(),
                        success: false,
                        output: String::new(),
                        error: Some(
                            result
                                .error
                                .unwrap_or_else(|| format!("Fallo en tool: {}", tool_name)),
                        ),
                        kind: "direct_tool".into(),
                    });
                }
            }

            Ok(QuickCheckResult {
                id: qc.id.clone(),
                label: qc.label.clone(),
                success: true,
                output: outputs.join("\n---\n"),
                error: None,
                kind: "direct_tool".into(),
            })
        }
        QuickCheckType::LlmPrompt => {
            // Devuelve el prompt para que el frontend lo inyecte en el chat
            Ok(QuickCheckResult {
                id: qc.id.clone(),
                label: qc.label.clone(),
                success: true,
                output: qc.prompt.clone().unwrap_or_default(),
                error: None,
                kind: "llm_prompt".into(),
            })
        }
    }
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn run_quick_check_compat(
    app: tauri::AppHandle,
    router: State<'_, Arc<AiRouter>>,
    quick_check_id: Option<String>,
    quickCheckId: Option<String>,
) -> Result<crate::config::settings::QuickCheckResult, String> {
    let resolved = quick_check_id
        .or(quickCheckId)
        .ok_or_else(|| "API_VALIDATION: 'quick_check_id' es requerido".to_string())?;

    run_quick_check(app, router, resolved).await
}

#[tauri::command]
pub async fn core_emit_event(
    state: State<'_, Arc<Mutex<crate::core::CoreState>>>,
    topic: String,
    level: Option<String>,
    message: String,
    payload: Option<serde_json::Value>,
) -> Result<crate::core::CoreEvent, String> {
    common::validate_non_empty("topic", &topic)?;
    common::validate_non_empty("message", &message)?;
    common::validate_max_len("topic", &topic, 120)?;
    common::validate_max_len("message", &message, 2000)?;

    let mut guard = state
        .lock()
        .map_err(|_| "No se pudo bloquear CoreState".to_string())?;
    crate::core::emit_event(
        &mut guard,
        &topic,
        level.as_deref().unwrap_or("info"),
        &message,
        payload.unwrap_or_else(|| serde_json::json!({})),
    )
}

#[tauri::command]
pub async fn core_list_events(
    state: State<'_, Arc<Mutex<crate::core::CoreState>>>,
    limit: Option<usize>,
) -> Result<Vec<crate::core::CoreEvent>, String> {
    let guard = state
        .lock()
        .map_err(|_| "No se pudo bloquear CoreState".to_string())?;
    Ok(crate::core::list_events(&guard, limit.unwrap_or(100)))
}

#[tauri::command]
pub async fn core_enqueue_task(
    state: State<'_, Arc<Mutex<crate::core::CoreState>>>,
    name: String,
    args: Option<serde_json::Value>,
) -> Result<crate::core::QueuedTask, String> {
    common::validate_non_empty("name", &name)?;
    common::validate_max_len("name", &name, 120)?;

    let mut guard = state
        .lock()
        .map_err(|_| "No se pudo bloquear CoreState".to_string())?;
    crate::core::enqueue_task(
        &mut guard,
        &name,
        args.unwrap_or_else(|| serde_json::json!({})),
    )
}

#[tauri::command]
pub async fn core_list_tasks(
    state: State<'_, Arc<Mutex<crate::core::CoreState>>>,
) -> Result<Vec<crate::core::QueuedTask>, String> {
    let guard = state
        .lock()
        .map_err(|_| "No se pudo bloquear CoreState".to_string())?;
    Ok(crate::core::list_tasks(&guard))
}

#[tauri::command]
pub async fn core_watchdog_heartbeat() -> Result<crate::core::WatchdogState, String> {
    crate::core::watchdog_heartbeat(env!("CARGO_PKG_VERSION"))
}

#[tauri::command]
pub async fn core_watchdog_status() -> Result<crate::core::WatchdogState, String> {
    crate::core::watchdog_status()
}

#[tauri::command]
pub async fn core_watchdog_health(
    seconds_threshold: Option<i64>,
) -> Result<serde_json::Value, String> {
    crate::core::watchdog_health(seconds_threshold.unwrap_or(60))
}

#[tauri::command]
pub async fn core_create_system_snapshot(
    source: Option<String>,
) -> Result<crate::core::SystemSnapshot, String> {
    let raw = crate::tools::sysinfo_tool::get_system_info_json();
    let data: serde_json::Value =
        serde_json::from_str(&raw).unwrap_or_else(|_| serde_json::json!({}));
    crate::core::create_snapshot(source.as_deref().unwrap_or("manual"), data)
}

#[tauri::command]
pub async fn core_list_system_snapshots(
    limit: Option<usize>,
) -> Result<Vec<crate::core::SystemSnapshot>, String> {
    Ok(crate::core::list_snapshots(limit.unwrap_or(50)))
}

#[tauri::command]
pub async fn core_recovery_check() -> Result<serde_json::Value, String> {
    crate::core::recovery_check()
}

#[tauri::command]
pub async fn core_set_recovery_mode(
    enabled: bool,
    reason: Option<String>,
) -> Result<crate::core::RecoveryState, String> {
    crate::core::set_recovery_mode(enabled, reason.as_deref().unwrap_or("manual"))
}

#[tauri::command]
pub async fn core_get_recovery_mode() -> Result<crate::core::RecoveryState, String> {
    crate::core::recovery_state()
}

#[tauri::command]
pub async fn core_save_dynamic_config_json(data: serde_json::Value) -> Result<bool, String> {
    crate::core::save_dynamic_config_json(&data)?;
    Ok(true)
}

#[tauri::command]
pub async fn core_load_dynamic_config_json() -> Result<serde_json::Value, String> {
    crate::core::load_dynamic_config_json()
}

#[tauri::command]
pub async fn core_save_dynamic_config_yaml(content: String) -> Result<bool, String> {
    crate::core::save_dynamic_config_yaml(&content)?;
    Ok(true)
}

#[tauri::command]
pub async fn core_load_dynamic_config_yaml() -> Result<String, String> {
    crate::core::load_dynamic_config_yaml()
}

#[tauri::command]
pub async fn core_process_queue_once(
    app: tauri::AppHandle,
    state: State<'_, Arc<Mutex<crate::core::CoreState>>>,
) -> Result<bool, String> {
    crate::core::process_queue_once(&app, &state).await;
    Ok(true)
}
