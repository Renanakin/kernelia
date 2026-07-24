use super::models::*;
use super::specialty_agent;
use super::tool_verifier;
use crate::config::AppSettings;
use crate::tools::ToolEngine;
use futures_util::StreamExt;
use std::sync::Arc;

const LOCAL_HISTORY_LIMIT: usize = 12;
const TOOL_SHORTLIST_LIMIT: usize = 12;

fn build_tool_shortlist(last_user_message: &str) -> Vec<crate::tools::ToolDefinition> {
    let definitions = ToolEngine::get_tool_definitions();
    let priority_names = specialty_agent::preferred_tools_for_message(last_user_message);

    let mut selected: Vec<crate::tools::ToolDefinition> = Vec::new();

    for name in priority_names {
        if let Some(def) = definitions.iter().find(|d| d.name == name) {
            selected.push(def.clone());
        }
        if selected.len() >= TOOL_SHORTLIST_LIMIT {
            break;
        }
    }

    if selected.is_empty() {
        for def in definitions.into_iter().take(TOOL_SHORTLIST_LIMIT) {
            selected.push(def);
        }
    }

    selected
}

/// Normaliza el historial de mensajes para modelos locales (Gemma, Llama, etc.)
/// 1. Extrae y unifica el System Prompt.
/// 2. Inyecta definiciones de herramientas en el System Prompt.
/// 3. Fusiona el System Prompt dentro del primer mensaje de User (crÃ­tico para evitar errores de paridad en Jinja).
/// 4. Convierte mensajes de Tool a User con marcadores claros.
/// 5. Asegura alternancia estricta User -> Assistant -> User.
fn normalize_messages_for_local(messages: &[ChatMessage], include_tools: bool) -> Vec<ChatMessage> {
    if messages.is_empty() {
        return Vec::new();
    }

    // 1. Recopilar System Prompt y definiciones de herramientas
    let mut system_content = String::new();
    let mut other_messages = Vec::new();

    for msg in messages {
        if msg.role == MessageRole::System {
            if !system_content.is_empty() {
                system_content.push_str("\n\n");
            }
            system_content.push_str(&msg.content);
        } else {
            other_messages.push(msg.clone());
        }
    }

    if other_messages.len() > LOCAL_HISTORY_LIMIT {
        let keep_from = other_messages.len().saturating_sub(LOCAL_HISTORY_LIMIT);
        other_messages = other_messages.into_iter().skip(keep_from).collect();
    }

    if include_tools {
        let last_user_message = other_messages
            .iter()
            .rev()
            .find(|m| m.role == MessageRole::User)
            .map(|m| m.content.clone())
            .unwrap_or_default();
        let tools = build_tool_shortlist(&last_user_message);
        if !tools.is_empty() {
            system_content.push_str("\n\n[INSTRUCCIONES TÃ‰CNICAS - HERRAMIENTAS]:\n");
            system_content
                .push_str("Responde con JSON de una sola herramienta cuando sea necesario.\n");
            system_content.push_str("Formato: {\"name\":\"tool_name\",\"arguments\":{...}}\n");
            system_content.push_str("Herramientas relevantes para ESTA consulta:\n");
            for t in tools {
                system_content.push_str(&format!("- {}: {}\n", t.name, t.description));
            }
            system_content.push_str(
                "Si usas herramienta, devuelve solo el JSON de llamada sin texto extra.\n",
            );
        }
    }

    // 2. Pre-procesar: Convertir Tool -> User y fusionar consecutivos del mismo rol
    let mut fused = Vec::new();
    for msg in other_messages {
        let (role, content) = match msg.role {
            MessageRole::Tool => (MessageRole::User, format!("\n[RESULTADO DE LA HERRAMIENTA]:\n{}\n---\nEste es el resultado de la herramienta que solicitaste. NO vuelvas a ejecutar la misma herramienta. Por favor, lee esta informaciÃ³n y explÃ­casela al usuario en lenguaje natural, claro y conciso.", msg.content)),
            _ => (msg.role.clone(), msg.content.clone()),
        };

        let content = if content.trim().is_empty() {
            match role {
                MessageRole::Assistant => "[Procesando...]".to_string(),
                _ => "(Sin contenido)".to_string(),
            }
        } else {
            content
        };

        if let Some(last) = fused.last_mut() {
            let last_msg: &mut ChatMessage = last;
            if last_msg.role == role {
                last_msg.content.push_str("\n\n");
                last_msg.content.push_str(&content);
                continue;
            }
        }

        fused.push(ChatMessage {
            role,
            content,
            reasoning_content: None,
            tool_call_id: None,
            tool_calls: None, // Siempre limpiar tool_calls para local para evitar errores de paridad
        });
    }

    // 3. Garantizar que el primer mensaje sea User e incluya el System Prompt
    let mut final_messages = Vec::new();

    if fused.is_empty() {
        // Solo hay system prompt o nada
        final_messages.push(ChatMessage {
            role: MessageRole::User,
            content: format!("[SISTEMA]: {}\n\nHola, KERNEL IA.", system_content),
            reasoning_content: None,
            tool_call_id: None,
            tool_calls: None,
        });
    } else {
        // Asegurar que el primero sea User
        if fused[0].role != MessageRole::User {
            final_messages.push(ChatMessage {
                role: MessageRole::User,
                content: "Hola. Por favor presÃ©ntate y prepÃ¡rate para ayudarme.".to_string(),
                reasoning_content: None,
                tool_call_id: None,
                tool_calls: None,
            });
        }

        // Agregar todos los fused
        final_messages.extend(fused);

        // Inyectar System Prompt en el primer mensaje de User
        if !system_content.is_empty() {
            if let Some(first_user) = final_messages
                .iter_mut()
                .find(|m| m.role == MessageRole::User)
            {
                let old = first_user.content.clone();
                first_user.content = format!(
                    "[INSTRUCCIONES DE SISTEMA]:\n{}\n\n[MENSAJE]:\n{}",
                    system_content, old
                );
            }
        }
    }

    // 4. VerificaciÃ³n final de alternancia (User -> Assistant -> User)
    let mut strictly_alternated: Vec<ChatMessage> = Vec::new();
    for msg in final_messages {
        if let Some(last) = strictly_alternated.last_mut() {
            let last_role: MessageRole = last.role.clone();
            if last_role == msg.role {
                // Esto no deberÃ­a pasar por el fused anterior, pero por seguridad fusionamos
                last.content.push_str("\n\n");
                last.content.push_str(&msg.content);
            } else {
                strictly_alternated.push(msg);
            }
        } else {
            strictly_alternated.push(msg);
        }
    }

    // 5. El Ãºltimo DEBE ser User para que el modelo responda
    if let Some(last) = strictly_alternated.last() {
        if last.role == MessageRole::Assistant {
            strictly_alternated.push(ChatMessage {
                role: MessageRole::User,
                content: "ContinÃºa con el diagnÃ³stico o genera tu conclusiÃ³n final.".to_string(),
                reasoning_content: None,
                tool_call_id: None,
                tool_calls: None,
            });
        }
    }

    strictly_alternated
}

/// Intenta extraer llamadas a herramientas de un texto (para modelos locales que no usan el campo tool_calls)
fn parse_tool_calls_from_text(text: &str) -> Vec<ToolCall> {
    let mut calls = Vec::new();

    // Buscar bloques de cÃ³digo JSON
    if let Some(start_idx) = text.find("```json") {
        let content = &text[start_idx + 7..];
        if let Some(end_idx) = content.find("```") {
            let json_str = content[..end_idx].trim();
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                if let Some(name) = val.get("name").and_then(|v| v.as_str()) {
                    let arguments = val
                        .get("arguments")
                        .map(|v| {
                            if v.is_string() {
                                v.as_str().unwrap().to_string()
                            } else {
                                v.to_string()
                            }
                        })
                        .unwrap_or_else(|| "{}".to_string());

                    calls.push(ToolCall {
                        id: Some(format!("call_{}", uuid::Uuid::new_v4().simple())),
                        call_type: Some("function".to_string()),
                        function: FunctionCall {
                            name: name.to_string(),
                            arguments,
                        },
                        index: Some(0),
                    });
                }
            }
        }
    }

    // Si no hay bloques de cÃ³digo, buscar JSON directamente
    if calls.is_empty() {
        if let Some(start_idx) = text.find('{') {
            if let Some(end_idx) = text.rfind('}') {
                let json_str = &text[start_idx..=end_idx];
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let Some(name) = val.get("name").and_then(|v| v.as_str()) {
                        let arguments = val
                            .get("arguments")
                            .map(|v| {
                                if v.is_string() {
                                    v.as_str().unwrap().to_string()
                                } else {
                                    v.to_string()
                                }
                            })
                            .unwrap_or_else(|| "{}".to_string());

                        calls.push(ToolCall {
                            id: Some(format!("call_{}", uuid::Uuid::new_v4().simple())),
                            call_type: Some("function".to_string()),
                            function: FunctionCall {
                                name: name.to_string(),
                                arguments,
                            },
                            index: Some(0),
                        });
                    }
                }
            }
        }
    }

    calls
}

/// Ejecuta el loop de function calling sincrÃ³nico
pub async fn function_calling_loop(
    app: &tauri::AppHandle,
    messages: &mut Vec<ChatMessage>,
    settings: &AppSettings,
) -> Result<FunctionCallingResult, String> {
    let client = super::client::AiClient::new(settings)?;
    let mut tools_used = Vec::new();
    let mut iteration = 0;
    let max_iterations = 5;
    let is_local = settings.selected_model.contains("local")
        || settings.selected_model.contains("gemma")
        || settings.selected_model.contains("llama");

    loop {
        if iteration >= max_iterations {
            break;
        }

        let request_messages = if is_local {
            normalize_messages_for_local(messages, true)
        } else {
            messages.clone()
        };

        let response = client.chat_completion(request_messages).await?;
        let choice = response.choices.first().ok_or("No response from AI")?;
        let msg = choice.message.clone();

        // Agregar respuesta al historial
        messages.push(msg.clone());

        let mut current_tool_calls = msg.tool_calls.clone();

        // Si no hay tool calls estructurados y es local, intentar extraer del texto
        if (current_tool_calls.is_none() || current_tool_calls.as_ref().unwrap().is_empty())
            && is_local
        {
            let extracted = parse_tool_calls_from_text(&msg.content);
            if !extracted.is_empty() {
                log::info!(
                    "Detected tool call in local AI text response (sync): {:?}",
                    extracted[0].function.name
                );
                current_tool_calls = Some(extracted);
            }
        }

        if let Some(tool_calls) = current_tool_calls {
            if tool_calls.is_empty() {
                return Ok(FunctionCallingResult {
                    response: msg.content,
                    tools_used,
                });
            }

            for call in tool_calls {
                let name = &call.function.name;
                let args_str = &call.function.arguments;
                let args: serde_json::Value = serde_json::from_str(args_str).unwrap_or_default();

                tools_used.push(ToolInfo {
                    name: name.clone(),
                    arguments: args_str.clone(),
                });

                // Ejecutar herramienta
                if crate::tools::rbac::is_owner_only_tool(name) && !settings.is_megaboss_unlocked()
                {
                    let msg = "Privilegio MegaBoss requerido para esta herramienta.";
                    messages.push(ChatMessage {
                        role: MessageRole::Tool,
                        content: msg.to_string(),
                        reasoning_content: None,
                        tool_call_id: call.id.clone(),
                        tool_calls: None,
                    });
                    continue;
                }
                let result = ToolEngine::execute(app, name, &args, settings.user_role).await;
                let tool_message_content = if result.success {
                    result.output.clone()
                } else {
                    result
                        .error
                        .clone()
                        .unwrap_or_else(|| "Error desconocido".to_string())
                };

                // Agregar resultado al historial
                messages.push(ChatMessage {
                    role: MessageRole::Tool,
                    content: tool_message_content,
                    reasoning_content: None,
                    tool_call_id: call.id.clone(),
                    tool_calls: None,
                });

                let verification = tool_verifier::verify_tool_execution(
                    app,
                    name,
                    &args,
                    &result,
                    settings,
                )
                .await;
                let verification_message =
                    tool_verifier::format_verification_message(name, &verification);
                messages.push(ChatMessage {
                    role: MessageRole::Tool,
                    content: verification_message,
                    reasoning_content: None,
                    tool_call_id: call.id.clone(),
                    tool_calls: None,
                });
            }
        } else {
            return Ok(FunctionCallingResult {
                response: msg.content,
                tools_used,
            });
        }

        iteration += 1;
    }

    Ok(FunctionCallingResult {
        response: messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default(),
        tools_used,
    })
}

/// Ejecuta el loop de function calling con streaming al frontend
pub async fn stream_function_calling_loop(
    app: &tauri::AppHandle,
    messages: &mut Vec<ChatMessage>,
    settings: &AppSettings,
    on_update: Arc<dyn Fn(StreamUpdate) + Send + Sync + 'static>,
) -> Result<FunctionCallingResult, String> {
    let client = super::client::AiClient::new(settings)?;
    let mut tools_used = Vec::new();
    let mut full_content = String::new();
    let mut iteration = 0;
    let max_iterations = 5;
    let is_local = settings.selected_model.contains("local")
        || settings.selected_model.contains("gemma")
        || settings.selected_model.contains("llama");

    loop {
        if iteration >= max_iterations {
            break;
        }

        if iteration == 0 {
            on_update(StreamUpdate {
                update_type: "text".to_string(),
                content: "ðŸ” [KERNEL IA]: Iniciando proceso de diagnÃ³stico integral...\n"
                    .to_string(),
                tool_name: None,
                tool_result: None,
            });
        }

        let request_messages = if is_local {
            normalize_messages_for_local(messages, true)
        } else {
            messages.clone()
        };

        if iteration > 0 {
            on_update(StreamUpdate {
                update_type: "text".to_string(),
                content: "\n\nðŸ§  [KERNEL IA]: Analizando resultados de las herramientas...\n"
                    .to_string(),
                tool_name: None,
                tool_result: None,
            });
        } else {
            on_update(StreamUpdate {
                update_type: "text".to_string(),
                content: "ðŸ“¡ [KERNEL IA]: Consultando a la inteligencia local...\n".to_string(),
                tool_name: None,
                tool_result: None,
            });
        }

        let mut stream = match client.chat_stream(request_messages).await {
            Ok(s) => s,
            Err(e) => {
                log::error!("Error starting chat stream: {}", e);
                return Err(e);
            }
        };

        full_content = String::new();
        let mut tool_calls_buffer: Vec<ToolCall> = Vec::new();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    if let Some(choice) = chunk.choices.first() {
                        // Docker Model Runner/Gemma4 puede emitir texto en reasoning_content.
                        let delta_text = choice
                            .delta
                            .content
                            .as_deref()
                            .filter(|content| !content.trim().is_empty())
                            .or(choice.delta.reasoning_content.as_deref());

                        // Procesar contenido de texto
                        if let Some(content) = delta_text {
                            full_content.push_str(content);
                            on_update(StreamUpdate {
                                update_type: "text".to_string(),
                                content: content.to_string(),
                                tool_name: None,
                                tool_result: None,
                            });
                        }

                        // Procesar tool calls parciales
                        if let Some(delta_tool_calls) = &choice.delta.tool_calls {
                            for delta_call in delta_tool_calls {
                                let idx = delta_call.index.unwrap_or(0) as usize;

                                while tool_calls_buffer.len() <= idx {
                                    tool_calls_buffer.push(ToolCall {
                                        id: None,
                                        call_type: Some("function".to_string()),
                                        function: FunctionCall::default(),
                                        index: Some(idx as u32),
                                    });
                                }

                                let call = &mut tool_calls_buffer[idx];
                                if let Some(id) = &delta_call.id {
                                    call.id = Some(id.clone());
                                }
                                call.function.name.push_str(&delta_call.function.name);
                                call.function
                                    .arguments
                                    .push_str(&delta_call.function.arguments);
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Error in chat stream chunk: {}", e);
                    break;
                }
            }
        }

        // Agregar el mensaje completo al historial
        let assistant_msg = ChatMessage {
            role: MessageRole::Assistant,
            content: full_content.clone(),
            reasoning_content: None,
            tool_call_id: None,
            tool_calls: if tool_calls_buffer.is_empty() {
                None
            } else {
                Some(tool_calls_buffer.clone())
            },
        };
        messages.push(assistant_msg);

        // Si no hay tool calls estructurados, intentamos extraer del texto (para modelos locales)
        if tool_calls_buffer.is_empty() && is_local {
            tool_calls_buffer = parse_tool_calls_from_text(&full_content);
            if !tool_calls_buffer.is_empty() {
                log::info!(
                    "Detected tool call in local AI text response: {:?}",
                    tool_calls_buffer[0].function.name
                );
            }
        }

        // Si no hay tool calls, terminamos el loop principal
        if tool_calls_buffer.is_empty() {
            return Ok(FunctionCallingResult {
                response: full_content,
                tools_used,
            });
        }

        // Procesar tool calls si se encontraron
        if !tool_calls_buffer.is_empty() {
            // Notificar que se estÃ¡n ejecutando herramientas
            on_update(StreamUpdate {
                update_type: "text".to_string(),
                content: format!("\n\nâš™ï¸ [KERNEL IA]: Ejecutando {} herramienta(s) de diagnÃ³stico en el sistema...\n", tool_calls_buffer.len()),
                tool_name: None,
                tool_result: None,
            });

            for call in tool_calls_buffer {
                let name = call.function.name;
                let args_str = call.function.arguments;
                let args: serde_json::Value = serde_json::from_str(&args_str).unwrap_or_default();

                on_update(StreamUpdate {
                    update_type: "tool_start".to_string(),
                    content: String::new(),
                    tool_name: Some(name.clone()),
                    tool_result: None,
                });

                // Ejecutar herramienta
                if crate::tools::rbac::is_owner_only_tool(&name) && !settings.is_megaboss_unlocked()
                {
                    let result_content =
                        "Privilegio MegaBoss requerido para esta herramienta.".to_string();
                    on_update(StreamUpdate {
                        update_type: "tool_end".to_string(),
                        content: "".to_string(),
                        tool_name: Some(name.clone()),
                        tool_result: Some(result_content.clone()),
                    });
                    messages.push(ChatMessage {
                        role: MessageRole::Tool,
                        content: result_content,
                        reasoning_content: None,
                        tool_call_id: call.id.clone(),
                        tool_calls: None,
                    });
                    continue;
                }
                let result = ToolEngine::execute(app, &name, &args, settings.user_role).await;
                let tool_message_content = if result.success {
                    result.output.clone()
                } else {
                    result
                        .error
                        .clone()
                        .unwrap_or_else(|| "Error desconocido".to_string())
                };

                tools_used.push(ToolInfo {
                    name: name.clone(),
                    arguments: args_str,
                });

                on_update(StreamUpdate {
                    update_type: "tool_end".to_string(),
                    content: String::new(),
                    tool_name: Some(name.clone()),
                    tool_result: Some(if result.success {
                        "OK".to_string()
                    } else {
                        "ERROR".to_string()
                    }),
                });

                // Agregar resultado al historial
                messages.push(ChatMessage {
                    role: MessageRole::Tool,
                    content: tool_message_content,
                    reasoning_content: None,
                    tool_call_id: call.id,
                    tool_calls: None,
                });

                let verification = tool_verifier::verify_tool_execution(
                    app,
                    &name,
                    &args,
                    &result,
                    settings,
                )
                .await;
                let verification_message =
                    tool_verifier::format_verification_message(&name, &verification);
                on_update(StreamUpdate {
                    update_type: "text".to_string(),
                    content: format!("\n{}\n", verification_message),
                    tool_name: None,
                    tool_result: None,
                });
                messages.push(ChatMessage {
                    role: MessageRole::Tool,
                    content: verification_message,
                    reasoning_content: None,
                    tool_call_id: None,
                    tool_calls: None,
                });
            }
        }

        iteration += 1;
    }

    Ok(FunctionCallingResult {
        response: full_content,
        tools_used,
    })
}

#[derive(Debug)]
pub struct FunctionCallingResult {
    pub response: String,
    pub tools_used: Vec<ToolInfo>,
}

#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub arguments: String,
}

#[cfg(test)]
mod tests {
    use super::{build_tool_shortlist, normalize_messages_for_local};
    use crate::ai::models::{ChatMessage, MessageRole};

    #[test]
    fn shortlist_for_network_query_is_bounded() {
        let tools = build_tool_shortlist("cuantas tarjetas de red tiene el equipo");
        assert!(!tools.is_empty());
        assert!(tools.len() <= 12);
        assert!(tools.iter().any(|t| t.name == "get_network_adapters"));
    }

    #[test]
    fn local_normalization_trims_long_history() {
        let mut msgs = Vec::new();
        msgs.push(ChatMessage {
            role: MessageRole::System,
            content: "system".into(),
            reasoning_content: None,
            tool_call_id: None,
            tool_calls: None,
        });

        for i in 0..40 {
            msgs.push(ChatMessage {
                role: if i % 2 == 0 {
                    MessageRole::User
                } else {
                    MessageRole::Assistant
                },
                content: format!("msg-{i}"),
                reasoning_content: None,
                tool_call_id: None,
                tool_calls: None,
            });
        }

        let normalized = normalize_messages_for_local(&msgs, true);
        assert!(!normalized.is_empty());
        assert!(normalized.len() <= 13);
    }
}
