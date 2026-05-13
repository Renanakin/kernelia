use super::function_calling;
use super::intent_engine;
use super::models::*;
use crate::config::AppSettings;
use crate::tools::ToolEngine;
use std::sync::{Arc, Mutex};

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
}

fn has_explicit_shortcut_intent(user_message: &str) -> bool {
    let text = user_message.to_lowercase();
    text.starts_with("/tool")
        || text.starts_with("/quick")
        || text.contains("quick check")
        || text.contains("ejecuta")
        || text.contains("run ")
}

fn detect_fast_text_response(user_message: &str) -> Option<&'static str> {
    let text = user_message
        .trim()
        .to_lowercase()
        .replace(',', "")
        .replace('.', "");
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");

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
    let text = user_message.to_lowercase();
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
            || text.contains("diagnóstico de red")
            || text.contains("conexion a internet")
            || text.contains("conexión a internet"))
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

    if text.contains("health")
        || text.contains("salud del equipo")
        || text.contains("salud del sistema")
        || text.contains("estado del equipo")
        || text.contains("estado del sistema")
    {
        return Some((
            "health_summary",
            serde_json::json!({}),
            "",
            "No se pudo obtener el health del equipo. ",
        ));
    }

    None
}
impl AiRouter {
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

    fn build_intent_context_message(&self, user_message: &str) -> ChatMessage {
        let recent = self.recent_context_window(6);
        let analysis = intent_engine::analyze_message(user_message, &recent);
        let operational_context = intent_engine::to_operational_context(&analysis);
        ChatMessage {
            role: MessageRole::System,
            content: operational_context,
            reasoning_content: None,
            tool_call_id: None,
            tool_calls: None,
        }
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
                error: None,
            },
            tool_name.to_string(),
        ))
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
        }
    }

    /// EnvÃƒÂ­a un mensaje del usuario y obtiene la respuesta del modelo
    pub async fn send_message(
        &self,
        app: &tauri::AppHandle,
        user_message: &str,
    ) -> Result<ChatResponse, String> {
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
                error: None,
            });
        }

        if let Some((shortcut_response, _tool_name)) =
            self.try_local_shortcut(app, user_message).await
        {
            return Ok(shortcut_response);
        }

        let mut messages = {
            let h = self.history.lock().map_err(|e| e.to_string())?;
            h.clone()
        };
        messages.push(self.build_intent_context_message(user_message));

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
                let tools_used: Vec<ToolUseInfo> = fc_result
                    .tools_used
                    .into_iter()
                    .map(|t| ToolUseInfo {
                        name: t.name,
                        arguments: t.arguments,
                    })
                    .collect();
                Ok(ChatResponse {
                    text: fc_result.response,
                    tools_used,
                    model: settings.selected_model.clone(),
                    error: None,
                })
            }
            Err(e) => Ok(ChatResponse {
                text: String::new(),
                tools_used: vec![],
                model: settings.selected_model.clone(),
                error: Some(e),
            }),
        }
    }

    /// EnvÃƒÂ­a un mensaje del usuario y obtiene la respuesta del modelo en streaming
    pub async fn stream_message(
        &self,
        app: &tauri::AppHandle,
        user_message: &str,
        on_update: Arc<dyn Fn(StreamUpdate) + Send + Sync + 'static>,
    ) -> Result<ChatResponse, String> {
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

        let mut messages = {
            let h = self.history.lock().map_err(|e| e.to_string())?;
            h.clone()
        };
        messages.push(self.build_intent_context_message(user_message));

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
                let tools_used: Vec<ToolUseInfo> = fc_result
                    .tools_used
                    .into_iter()
                    .map(|t| ToolUseInfo {
                        name: t.name,
                        arguments: t.arguments,
                    })
                    .collect();
                Ok(ChatResponse {
                    text: fc_result.response,
                    tools_used,
                    model: settings.selected_model.clone(),
                    error: None,
                })
            }
            Err(e) => Ok(ChatResponse {
                text: String::new(),
                tools_used: vec![],
                model: settings.selected_model.clone(),
                error: Some(e),
            }),
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
    use super::{detect_fast_text_response, detect_local_shortcut};

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
        let q = "cuantas unidades de almacenamiento tiene el equipo";
        let shortcut = detect_local_shortcut(q).expect("storage shortcut should be detected");
        assert_eq!(shortcut.0, "get_storage_summary");
    }

    #[test]
    fn detects_health_question_as_local_tool() {
        let q = "cual es el health del equipo";
        let shortcut = detect_local_shortcut(q).expect("health shortcut should be detected");
        assert_eq!(shortcut.0, "health_summary");
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
}

/// Respuesta del chat hacia el frontend
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatResponse {
    pub text: String,
    pub tools_used: Vec<ToolUseInfo>,
    pub model: String,
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
