mod ai;
mod commands;
mod config;
mod core;
mod rag;
mod tools;

use ai::AiRouter;
use config::AppSettings;
use std::sync::Arc;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    // Cargar .env desde la raiz del proyecto (API keys protegidas)
    load_dotenv();

    // Cargar configuracion (portable: junto al .exe)
    let mut settings = AppSettings::load();

    // Inyectar API keys desde variables de entorno si no hay keys guardadas
    settings.inject_env_keys();

    log::info!("KERNEL IA v{} starting...", settings.version);
    log::info!("Selected model: {}", settings.selected_model);
    log::info!("Config path: {:?}", AppSettings::config_path());

    // Crear el router de IA
    let router = Arc::new(AiRouter::new(settings));
    let mut core_state = core::CoreState::new();
    core::load_tasks(&mut core_state);
    let core_state = Arc::new(Mutex::new(core_state));

    let core_state_for_setup = core_state.clone();

    tauri::Builder::default()
        .setup(move |app| {
            // Hilo dedicado para el programador de tareas para evitar problemas con el reactor de Tokio
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let rt = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(rt) => rt,
                    Err(e) => {
                        log::error!(
                            "No se pudo crear el runtime de Tokio para el scheduler: {}",
                            e
                        );
                        return;
                    }
                };

                rt.block_on(async move {
                    log::info!("Iniciando hilo del programador de tareas...");
                    loop {
                        crate::tools::scheduler::run_pending_tasks(&handle).await;
                        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                    }
                });
            });

            // Watchdog heartbeat periodico (no bloqueante)
            std::thread::spawn(|| loop {
                let _ = crate::core::watchdog_heartbeat(env!("CARGO_PKG_VERSION"));
                std::thread::sleep(std::time::Duration::from_secs(30));
            });

            // Snapshot periodico basico del estado del sistema
            std::thread::spawn(|| loop {
                let raw = crate::tools::sysinfo_tool::get_system_info_json();
                let data: serde_json::Value =
                    serde_json::from_str(&raw).unwrap_or_else(|_| serde_json::json!({}));
                let _ = crate::core::create_snapshot("periodic", data);
                std::thread::sleep(std::time::Duration::from_secs(300));
            });

            // Worker de cola de tareas
            let queue_handle = app.handle().clone();
            let queue_state = core_state_for_setup.clone();
            std::thread::spawn(move || {
                let rt = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(rt) => rt,
                    Err(e) => {
                        log::error!("No se pudo crear runtime para cola de tareas: {}", e);
                        return;
                    }
                };
                rt.block_on(async move {
                    loop {
                        crate::core::process_queue_once(&queue_handle, &queue_state).await;
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                });
            });

            // Recovery mode automatico si el watchdog esta stale al iniciar
            if let Ok(h) = crate::core::watchdog_health(120) {
                if h.get("status").and_then(|v| v.as_str()) == Some("stale") {
                    let _ =
                        crate::core::set_recovery_mode(true, "watchdog_stale_detected_on_startup");
                } else {
                    let _ = crate::core::set_recovery_mode(false, "startup_ok");
                }
            }
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .manage(router)
        .manage(core_state)
        .invoke_handler(tauri::generate_handler![
            // Chat commands
            commands::chat::send_message,
            commands::chat::stream_message,
            commands::chat::analyze_intent,
            commands::chat::clear_chat,
            commands::chat::set_model,
            commands::chat::set_model_compat,
            commands::chat::set_api_key,
            commands::chat::set_api_key_compat,
            commands::chat::get_settings,
            commands::chat::update_settings,
            commands::chat::detect_ollama,
            commands::chat::get_models,
            commands::chat::set_megaboss_password,
            commands::chat::unlock_megaboss,
            commands::chat::megaboss_status,
            commands::chat::login_user,
            commands::chat::logout_user,
            commands::chat::get_auth_status,
            commands::chat::unlock_tecnico_critical,
            commands::chat::list_support_users,
            commands::chat::create_support_user,
            commands::chat::delete_support_user,
            commands::chat::confirm_solution_and_ingest,
            commands::chat::create_support_ticket_cmd,
            commands::chat::list_support_tickets_cmd,
            commands::chat::create_hitl_checkpoint_cmd,
            commands::chat::resolve_hitl_checkpoint_cmd,
            commands::chat::list_pending_checkpoints_cmd,
            commands::chat::verify_tech_password_cmd,
            commands::chat::log_user_interaction_cmd,
            // System commands
            commands::system::get_system_info,
            commands::system::list_processes,
            commands::system::list_processes_compat,
            commands::system::get_app_version,
            commands::system::run_network_diagnostic,
            commands::system::list_running_services,
            commands::system::restart_service,
            commands::system::generate_support_report,
            commands::system::get_audit_logs,
            commands::system::execute_tool,
            commands::system::get_quick_checks,
            commands::system::run_quick_check,
            commands::system::run_quick_check_compat,
            commands::system::core_emit_event,
            commands::system::core_list_events,
            commands::system::core_enqueue_task,
            commands::system::core_list_tasks,
            commands::system::core_watchdog_heartbeat,
            commands::system::core_watchdog_status,
            commands::system::core_watchdog_health,
            commands::system::core_create_system_snapshot,
            commands::system::core_list_system_snapshots,
            commands::system::core_recovery_check,
            commands::system::core_set_recovery_mode,
            commands::system::core_get_recovery_mode,
            commands::system::core_save_dynamic_config_json,
            commands::system::core_load_dynamic_config_json,
            commands::system::core_save_dynamic_config_yaml,
            commands::system::core_load_dynamic_config_yaml,
            commands::system::core_process_queue_once,
        ])
        .run(tauri::generate_context!())
        .expect("error while running KERNEL IA");
}

/// Carga el archivo .env buscando en la raiz del proyecto o junto al ejecutable
fn load_dotenv() {
    // Intenta desde el directorio de trabajo actual (dev mode)
    if dotenvy::dotenv().is_ok() {
        log::info!(".env loaded from working directory");
        return;
    }
    // Intenta junto al ejecutable (release/portable)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let env_path = dir.join(".env");
            if env_path.exists() {
                if dotenvy::from_path(&env_path).is_ok() {
                    log::info!(".env loaded from {:?}", env_path);
                    return;
                }
            }
        }
    }
    log::info!("No .env file found (keys can be set from Settings)");
}
