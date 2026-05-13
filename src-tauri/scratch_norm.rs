use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub tool_call_id: Option<String>,
    pub tool_calls: Option<Vec<String>>, // mock
}

pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: String,
}

struct ToolEngine;
impl ToolEngine {
    pub fn get_tool_definitions() -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "run_network_diagnostic".into(),
                description: "Ejecuta un diagnostico completo de red.".into(),
                parameters: "{}".into(),
            },
        ]
    }
}

fn normalize_messages_for_local(messages: &[ChatMessage], include_tools: bool) -> Vec<ChatMessage> {
    if messages.is_empty() {
        return Vec::new();
    }

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

    if include_tools {
        let tools = ToolEngine::get_tool_definitions();
        if !tools.is_empty() {
            system_content.push_str("\n\n[INSTRUCCIONES TÉCNICAS - HERRAMIENTAS]:\n");
            system_content.push_str("Para usar una herramienta, responde ÚNICAMENTE con el bloque JSON de la función dentro de etiquetas ```json. Ejemplo:\n");
            system_content.push_str("```json\n{\n  \"name\": \"run_network_diagnostic\",\n  \"arguments\": \"{}\"\n}\n```\n");
            system_content.push_str("Herramientas disponibles:\n");
            for t in tools {
                system_content.push_str(&format!("- {}: {}\n  Argumentos esperados: {}\n", t.name, t.description, t.parameters));
            }
            system_content.push_str("\nSi decides usar una herramienta, detén tu respuesta inmediatamente después del bloque JSON.");
        }
    }

    let mut fused = Vec::new();
    for msg in other_messages {
        let (role, content) = match msg.role {
            MessageRole::Tool => (MessageRole::User, format!("\n[RESULTADO DE LA HERRAMIENTA]:\n{}\n---\nEste es el resultado de la herramienta que solicitaste. NO vuelvas a ejecutar la misma herramienta. Por favor, lee esta información y explícasela al usuario en lenguaje natural, claro y conciso.", msg.content)),
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
            tool_call_id: None,
            tool_calls: None, 
        });
    }

    let mut final_messages = Vec::new();
    
    if fused.is_empty() {
        final_messages.push(ChatMessage {
            role: MessageRole::User,
            content: format!("[SISTEMA]: {}\n\nHola, KERNEL IA.", system_content),
            tool_call_id: None,
            tool_calls: None,
        });
    } else {
        if fused[0].role != MessageRole::User {
            final_messages.push(ChatMessage {
                role: MessageRole::User,
                content: "Hola. Por favor preséntate y prepárate para ayudarme.".to_string(),
                tool_call_id: None,
                tool_calls: None,
            });
        }
        
        final_messages.extend(fused);
        
        if !system_content.is_empty() {
            if let Some(first_user) = final_messages.iter_mut().find(|m| m.role == MessageRole::User) {
                let old = first_user.content.clone();
                first_user.content = format!("[INSTRUCCIONES DE SISTEMA]:\n{}\n\n[MENSAJE]:\n{}", system_content, old);
            }
        }
    }

    let mut strictly_alternated: Vec<ChatMessage> = Vec::new();
    for msg in final_messages {
        if let Some(last) = strictly_alternated.last_mut() {
            let last_role: MessageRole = last.role.clone();
            if last_role == msg.role {
                last.content.push_str("\n\n");
                last.content.push_str(&msg.content);
            } else {
                strictly_alternated.push(msg);
            }
        } else {
            strictly_alternated.push(msg);
        }
    }

    if let Some(last) = strictly_alternated.last() {
        if last.role == MessageRole::Assistant {
            strictly_alternated.push(ChatMessage {
                role: MessageRole::User,
                content: "Continúa con el diagnóstico o genera tu conclusión final.".to_string(),
                tool_call_id: None,
                tool_calls: None,
            });
        }
    }

    strictly_alternated
}

fn main() {
    let mut messages = vec![
        ChatMessage {
            role: MessageRole::System,
            content: "Eres KERNEL IA, un asistente. No inventes comandos.".to_string(),
            tool_call_id: None,
            tool_calls: None,
        },
        ChatMessage {
            role: MessageRole::User,
            content: "EJECUTA LA REVISION EN OPERACION".to_string(),
            tool_call_id: None,
            tool_calls: None,
        }
    ];

    let norm = normalize_messages_for_local(&messages, true);
    for m in norm {
        println!("ROLE: {:?}", m.role);
        println!("CONTENT: \n{}", m.content);
        println!("--------------------------------------------------");
    }
}
