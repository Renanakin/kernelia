use serde::{Deserialize, Serialize};

/// Rol de un mensaje en la conversación
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// Un mensaje en la conversación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    #[serde(default, deserialize_with = "deserialize_content")]
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// Deserializa content como String, convirtiendo null/missing en ""
fn deserialize_content<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

/// Llamada a una herramienta solicitada por el modelo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(rename = "type", default)]
    pub call_type: Option<String>,
    #[serde(default)]
    pub function: FunctionCall,
    #[serde(default)]
    pub index: Option<u32>,
}

/// Detalles de la función a ejecutar
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FunctionCall {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub arguments: String,
}

/// Request body para la API estilo OpenAI
#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolSpec>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
}

/// Especificación de tool para la API
#[derive(Debug, Clone, Serialize)]
pub struct ToolSpec {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolFunction,
}

/// Definición de función para la API
#[derive(Debug, Clone, Serialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Respuesta de la API
#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: Option<String>,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

/// Una opción de respuesta
#[derive(Debug, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

/// Uso de tokens
#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

/// Chunk de respuesta en streaming
#[derive(Debug, Deserialize)]
pub struct ChatCompletionChunk {
    pub id: Option<String>,
    pub choices: Vec<DeltaChoice>,
}

/// Una opción de chunk en streaming
#[derive(Debug, Deserialize)]
pub struct DeltaChoice {
    pub delta: Delta,
    pub index: u32,
    pub finish_reason: Option<String>,
}

/// El contenido parcial en un chunk
#[derive(Debug, Deserialize)]
pub struct Delta {
    pub role: Option<MessageRole>,
    pub content: Option<String>,
    pub reasoning_content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// Respuesta de modelos de Ollama
#[derive(Debug, Deserialize)]
pub struct OllamaTagsResponse {
    pub models: Option<Vec<OllamaModel>>,
}

/// Modelo disponible en Ollama
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OllamaModel {
    pub name: String,
    pub size: Option<u64>,
    pub modified_at: Option<String>,
}

/// Mensaje enviado al frontend con información de progreso
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamUpdate {
    #[serde(rename = "type")]
    pub update_type: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_result: Option<String>,
}
