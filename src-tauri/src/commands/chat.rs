use crate::ai::intent_engine::{self, IntentAnalysis};
use crate::ai::models::StreamUpdate;
use crate::ai::router::{AiRouter, ChatResponse, OllamaInstance};
use crate::commands::common;
use crate::config::{AppSettings, SupportProfile};
use std::sync::Arc;
use tauri::State;

fn mask_username(username: &str) -> String {
    if username.len() <= 2 {
        "**".to_string()
    } else {
        format!("{}***", &username[..2])
    }
}

async fn refresh_technician_role_if_needed(
    router: &State<'_, Arc<AiRouter>>,
) -> Result<(), String> {
    let mut settings = router.get_settings()?;
    let before = settings.user_role;
    settings.refresh_tecnico_role_if_needed();
    if settings.user_role != before {
        router.update_settings(settings)?;
    }
    Ok(())
}

/// Envía un mensaje del usuario al modelo de IA y obtiene la respuesta
#[tauri::command]
pub async fn send_message(
    app: tauri::AppHandle,
    message: String,
    router: State<'_, Arc<AiRouter>>,
) -> Result<ChatResponse, String> {
    common::validate_message(&message)?;
    let s = router.get_settings()?;
    if !s.auth.is_authenticated {
        return Err("Debe iniciar sesion para usar KernelIA".to_string());
    }
    refresh_technician_role_if_needed(&router).await?;
    router.send_message(&app, &message).await
}

/// Envía un mensaje del usuario al modelo de IA y obtiene la respuesta en streaming
#[tauri::command]
pub async fn stream_message(
    app: tauri::AppHandle,
    message: String,
    channel: tauri::ipc::Channel<StreamUpdate>,
    router: State<'_, Arc<AiRouter>>,
) -> Result<ChatResponse, String> {
    common::validate_message(&message)?;
    let s = router.get_settings()?;
    if !s.auth.is_authenticated {
        return Err("Debe iniciar sesion para usar KernelIA".to_string());
    }
    refresh_technician_role_if_needed(&router).await?;
    router
        .stream_message(
            &app,
            &message,
            Arc::new(move |update| {
                let _ = channel.send(update);
            }),
        )
        .await
}

#[tauri::command]
pub async fn analyze_intent(
    message: String,
    router: State<'_, Arc<AiRouter>>,
) -> Result<IntentAnalysis, String> {
    common::validate_message(&message)?;
    let s = router.get_settings()?;
    if !s.auth.is_authenticated {
        return Err("Debe iniciar sesion para usar KernelIA".to_string());
    }
    let analysis = intent_engine::analyze_message(&message, &[]);
    Ok(analysis)
}

/// Limpia el historial de conversación
#[tauri::command]
pub async fn clear_chat(router: State<'_, Arc<AiRouter>>) -> Result<(), String> {
    router.clear_history()
}

/// Cambia el modelo de IA seleccionado
#[tauri::command]
pub async fn set_model(model_id: String, router: State<'_, Arc<AiRouter>>) -> Result<(), String> {
    common::validate_model_id(&model_id)?;
    router.set_model(&model_id)
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn set_model_compat(
    model_id: Option<String>,
    modelId: Option<String>,
    router: State<'_, Arc<AiRouter>>,
) -> Result<(), String> {
    let resolved = model_id
        .or(modelId)
        .ok_or_else(|| "API_VALIDATION: 'model_id' es requerido".to_string())?;
    common::validate_model_id(&resolved)?;
    router.set_model(&resolved)
}

/// Establece la API key para un modelo
#[tauri::command]
pub async fn set_api_key(
    model_id: String,
    api_key: String,
    router: State<'_, Arc<AiRouter>>,
) -> Result<(), String> {
    common::validate_model_id(&model_id)?;
    common::validate_password(&api_key, "api_key")?;
    router.set_api_key(&model_id, &api_key)
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn set_api_key_compat(
    model_id: Option<String>,
    modelId: Option<String>,
    api_key: Option<String>,
    apiKey: Option<String>,
    router: State<'_, Arc<AiRouter>>,
) -> Result<(), String> {
    let resolved_model = model_id
        .or(modelId)
        .ok_or_else(|| "API_VALIDATION: 'model_id' es requerido".to_string())?;
    let resolved_key = api_key
        .or(apiKey)
        .ok_or_else(|| "API_VALIDATION: 'api_key' es requerido".to_string())?;

    common::validate_model_id(&resolved_model)?;
    common::validate_password(&resolved_key, "api_key")?;

    router.set_api_key(&resolved_model, &resolved_key)
}

/// Obtiene la configuración actual
#[tauri::command]
pub async fn get_settings(router: State<'_, Arc<AiRouter>>) -> Result<AppSettings, String> {
    router.get_settings()
}

/// Actualiza la configuración
#[tauri::command]
pub async fn update_settings(
    settings: AppSettings,
    router: State<'_, Arc<AiRouter>>,
) -> Result<(), String> {
    router.update_settings(settings)
}

/// Detecta instancias de Ollama disponibles (local y en red)
#[tauri::command]
pub async fn detect_ollama(
    router: State<'_, Arc<AiRouter>>,
) -> Result<Vec<OllamaInstance>, String> {
    Ok(router.detect_ollama().await)
}

/// Obtiene la lista de modelos disponibles
#[tauri::command]
pub async fn get_models(router: State<'_, Arc<AiRouter>>) -> Result<Vec<ModelInfo>, String> {
    let settings = router.get_settings()?;
    let models: Vec<ModelInfo> = settings
        .models
        .iter()
        .map(|m| ModelInfo {
            id: m.id.clone(),
            name: m.name.clone(),
            provider: m.provider.clone(),
            has_api_key: m.api_key.is_some(),
            is_local: m.is_local,
            selected: m.id == settings.selected_model,
        })
        .collect();
    Ok(models)
}

#[tauri::command]
pub async fn set_megaboss_password(
    password: String,
    router: State<'_, Arc<AiRouter>>,
) -> Result<(), String> {
    common::validate_password(&password, "password")?;
    let mut settings = router.get_settings()?;
    settings.set_megaboss_password(&password)?;
    router.update_settings(settings)
}

#[tauri::command]
pub async fn unlock_megaboss(
    password: String,
    minutes: Option<i64>,
    router: State<'_, Arc<AiRouter>>,
) -> Result<bool, String> {
    common::validate_password(&password, "password")?;
    let mut settings = router.get_settings()?;
    if !settings.verify_megaboss_password(&password) {
        return Ok(false);
    }
    let m = minutes.unwrap_or(20).clamp(1, 240);
    settings.unlock_megaboss_for_minutes(m)?;
    router.update_settings(settings)?;
    Ok(true)
}

#[tauri::command]
pub async fn megaboss_status(router: State<'_, Arc<AiRouter>>) -> Result<MegabossStatus, String> {
    let settings = router.get_settings()?;
    Ok(MegabossStatus {
        password_set: settings.megaboss_password_encrypted.is_some(),
        unlocked: settings.is_megaboss_unlocked(),
        unlock_until_epoch: settings.megaboss_unlock_until_epoch,
    })
}

/// Info de modelo simplificada para el frontend
#[derive(serde::Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub has_api_key: bool,
    pub is_local: bool,
    pub selected: bool,
}

#[derive(serde::Serialize)]
pub struct MegabossStatus {
    pub password_set: bool,
    pub unlocked: bool,
    pub unlock_until_epoch: Option<i64>,
}

#[derive(serde::Serialize)]
pub struct AuthStatus {
    pub is_authenticated: bool,
    pub username: Option<String>,
    pub profile: Option<SupportProfile>,
    pub role: String,
    pub tecnico_critical_unlocked: bool,
    pub tecnico_unlock_until_epoch: Option<i64>,
}

#[derive(serde::Serialize)]
pub struct SupportUserInfo {
    pub username: String,
    pub profile: SupportProfile,
    pub active: bool,
}

#[tauri::command]
pub async fn login_user(
    username: String,
    password: String,
    router: State<'_, Arc<AiRouter>>,
) -> Result<AuthStatus, String> {
    common::validate_username(&username)?;
    common::validate_password(&password, "password")?;
    let mut settings = router.get_settings()?;
    settings.login(&username, &password)?;
    router.update_settings(settings.clone())?;

    let now = chrono::Utc::now().timestamp();
    let tecnico_unlocked = settings
        .auth
        .tecnico_critical_unlock_until_epoch
        .map(|t| t > now)
        .unwrap_or(false);

    Ok(AuthStatus {
        is_authenticated: settings.auth.is_authenticated,
        username: settings
            .auth
            .current_username
            .clone()
            .map(|u| mask_username(&u)),
        profile: settings.auth.current_profile.clone(),
        role: format!("{:?}", settings.user_role),
        tecnico_critical_unlocked: tecnico_unlocked,
        tecnico_unlock_until_epoch: settings.auth.tecnico_critical_unlock_until_epoch,
    })
}

#[tauri::command]
pub async fn logout_user(router: State<'_, Arc<AiRouter>>) -> Result<(), String> {
    let mut settings = router.get_settings()?;
    settings.logout()?;
    router.update_settings(settings)
}

#[tauri::command]
pub async fn get_auth_status(router: State<'_, Arc<AiRouter>>) -> Result<AuthStatus, String> {
    let mut settings = router.get_settings()?;
    settings.refresh_tecnico_role_if_needed();
    router.update_settings(settings.clone())?;

    let now = chrono::Utc::now().timestamp();
    let tecnico_unlocked = settings
        .auth
        .tecnico_critical_unlock_until_epoch
        .map(|t| t > now)
        .unwrap_or(false);

    Ok(AuthStatus {
        is_authenticated: settings.auth.is_authenticated,
        username: settings
            .auth
            .current_username
            .clone()
            .map(|u| mask_username(&u)),
        profile: settings.auth.current_profile.clone(),
        role: format!("{:?}", settings.user_role),
        tecnico_critical_unlocked: tecnico_unlocked,
        tecnico_unlock_until_epoch: settings.auth.tecnico_critical_unlock_until_epoch,
    })
}

#[tauri::command]
pub async fn unlock_tecnico_critical(
    password: String,
    minutes: Option<i64>,
    router: State<'_, Arc<AiRouter>>,
) -> Result<bool, String> {
    common::validate_password(&password, "password")?;
    let mut settings = router.get_settings()?;
    if !settings.auth.is_authenticated
        || settings.auth.current_profile != Some(SupportProfile::Tecnico)
    {
        return Err("Esta accion solo aplica para sesiones Tecnico autenticadas".to_string());
    }
    if !settings.verify_tecnico_critical_password(&password) {
        return Ok(false);
    }
    let m = minutes.unwrap_or(15).clamp(1, 120);
    settings.unlock_tecnico_critical_for_minutes(m)?;
    router.update_settings(settings)?;
    Ok(true)
}

#[tauri::command]
pub async fn list_support_users(
    router: State<'_, Arc<AiRouter>>,
) -> Result<Vec<SupportUserInfo>, String> {
    let settings = router.get_settings()?;
    if !settings.is_superuser() {
        return Err("Solo el superusuario puede listar usuarios".to_string());
    }
    Ok(settings
        .users
        .iter()
        .map(|u| SupportUserInfo {
            username: u.username.clone(),
            profile: u.profile.clone(),
            active: u.active,
        })
        .collect())
}

#[tauri::command]
pub async fn create_support_user(
    username: String,
    password: String,
    profile: SupportProfile,
    router: State<'_, Arc<AiRouter>>,
) -> Result<(), String> {
    common::validate_username(&username)?;
    common::validate_password(&password, "password")?;
    let mut settings = router.get_settings()?;
    if !settings.auth.is_authenticated || settings.auth.current_profile != Some(SupportProfile::Superusuario) {
        return Err("ACCESO_DENEGADO: Requiere perfil de Superusuario autenticado.".to_string());
    }
    settings.create_user(&username, &password, profile)?;
    router.update_settings(settings)
}

#[tauri::command]
pub async fn delete_support_user(
    username: String,
    router: State<'_, Arc<AiRouter>>,
) -> Result<(), String> {
    common::validate_username(&username)?;
    let mut settings = router.get_settings()?;
    if !settings.auth.is_authenticated || settings.auth.current_profile != Some(SupportProfile::Superusuario) {
        return Err("ACCESO_DENEGADO: Requiere perfil de Superusuario autenticado.".to_string());
    }
    settings.delete_user(&username)?;
    router.update_settings(settings)
}

#[tauri::command]
pub async fn confirm_solution_and_ingest(
    query: String,
    solution: String,
    specialty: Option<String>,
) -> Result<String, String> {
    common::validate_non_empty("query", &query)?;
    common::validate_non_empty("solution", &solution)?;
    let spec = specialty.unwrap_or_else(|| "sp_general".to_string());
    crate::rag::auto_ingest::ingest_user_validated_solution(&query, &solution, &spec)
}

#[tauri::command]
pub async fn create_support_ticket_cmd(
    query: String,
    specialty: Option<String>,
    telemetry: Option<String>,
) -> Result<crate::rag::models::TicketCreationResult, String> {
    common::validate_non_empty("query", &query)?;
    let spec = specialty.unwrap_or_else(|| "General".to_string());
    let telem = telemetry.unwrap_or_else(|| "{}".to_string());
    crate::ai::ticket_agent::create_support_ticket_record(&query, &spec, &telem)
}

#[tauri::command]
pub async fn list_support_tickets_cmd() -> Result<Vec<crate::rag::models::SupportTicket>, String> {
    crate::ai::ticket_agent::list_support_tickets_from_db()
}
