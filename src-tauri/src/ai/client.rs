use crate::ai::models::*;
use crate::config::AppSettings;
use futures_util::{Stream, StreamExt};
use reqwest::{header::CONTENT_TYPE, Client};
use std::pin::Pin;
use std::time::Duration;

pub struct AiClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
    model_name: String,
    fallback_base_url: Option<String>,
    fallback_model_name: Option<String>,
    provider: String,
}

impl AiClient {
    pub fn new(settings: &AppSettings) -> Result<Self, String> {
        let model = settings.current_model().ok_or("No model selected")?;

        let api_key = if model.provider == "docker-model-runner" {
            None
        } else {
            settings.get_api_key(&model.id)
        };

        let (fallback_base_url, fallback_model_name) = if model.provider == "docker-model-runner"
            && (model.id == "gemma4-local" || model.model_name.to_lowercase().contains("gemma4"))
        {
            if let Some(g3) = settings.models.iter().find(|m| m.id == "gemma3-local") {
                (Some(g3.base_url.clone()), Some(g3.model_name.clone()))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        Ok(Self {
            client: Client::builder()
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(120))
                .build()
                .map_err(|e| format!("No se pudo crear cliente HTTP de IA: {}", e))?,
            base_url: model.base_url.clone(),
            api_key,
            model_name: model.model_name.clone(),
            fallback_base_url,
            fallback_model_name,
            provider: model.provider.clone(),
        })
    }

    fn local_generation_profile(&self) -> bool {
        self.provider == "docker-model-runner"
    }

    fn generation_max_tokens(&self) -> u32 {
        if self.local_generation_profile() {
            500
        } else {
            2048
        }
    }

    fn generation_temperature(&self) -> f32 {
        if self.local_generation_profile() {
            0.6
        } else {
            0.1
        }
    }

    fn generation_top_p(&self) -> Option<f32> {
        if self.local_generation_profile() {
            Some(0.9)
        } else {
            None
        }
    }

    async fn chat_completion_with_model(
        &self,
        base_url: &str,
        model_name: &str,
        messages: Vec<ChatMessage>,
    ) -> Result<ChatCompletionResponse, String> {
        let url = format!("{}/chat/completions", base_url);

        let request = ChatCompletionRequest {
            model: model_name.to_string(),
            messages,
            tools: None,
            tool_choice: None,
            max_tokens: Some(self.generation_max_tokens()),
            temperature: Some(self.generation_temperature()),
            top_p: self.generation_top_p(),
        };

        let mut req = self.client.post(&url).json(&request);

        if let Some(key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let resp = req.send().await.map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            log::error!("AI API Error: {}", err_text);
            return Err(format!("AI API Error ({}): {}", status, err_text));
        }

        let mut parsed = resp
            .json::<ChatCompletionResponse>()
            .await
            .map_err(|e| e.to_string())?;
        Self::normalize_reasoning_content(&mut parsed);
        Ok(parsed)
    }

    fn normalize_reasoning_content(resp: &mut ChatCompletionResponse) {
        for choice in &mut resp.choices {
            if choice.message.content.trim().is_empty() {
                if let Some(reasoning) = &choice.message.reasoning_content {
                    let trimmed = reasoning.trim();
                    if !trimmed.is_empty() {
                        choice.message.content = trimmed.to_string();
                    }
                }
            }
        }
    }

    fn response_to_stream_chunks(
        resp: ChatCompletionResponse,
    ) -> Vec<Result<ChatCompletionChunk, String>> {
        let id = resp.id;
        resp.choices
            .into_iter()
            .map(|choice| {
                Ok(ChatCompletionChunk {
                    id: id.clone(),
                    choices: vec![DeltaChoice {
                        delta: Delta {
                            role: Some(MessageRole::Assistant),
                            content: Some(choice.message.content),
                            reasoning_content: choice.message.reasoning_content,
                            tool_calls: choice.message.tool_calls,
                        },
                        index: choice.index,
                        finish_reason: choice.finish_reason,
                    }],
                })
            })
            .collect()
    }

    pub async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<ChatCompletionResponse, String> {
        // Log messages for debugging (local only)
        if self.provider == "docker-model-runner" {
            log::info!("Sending {} messages to local AI", messages.len());
            for (i, msg) in messages.iter().enumerate() {
                log::debug!(
                    "  [{}] {}: {} chars",
                    i,
                    serde_json::to_string(&msg.role).unwrap_or_default(),
                    msg.content.len()
                );
            }
        }

        let primary = self
            .chat_completion_with_model(&self.base_url, &self.model_name, messages.clone())
            .await;

        match primary {
            Ok(resp) => Ok(resp),
            Err(primary_err) => {
                if let (Some(fallback_url), Some(fallback_model)) =
                    (&self.fallback_base_url, &self.fallback_model_name)
                {
                    log::warn!(
                        "Primary model '{}' failed, trying fallback '{}': {}",
                        self.model_name,
                        fallback_model,
                        primary_err
                    );
                    self.chat_completion_with_model(fallback_url, fallback_model, messages)
                        .await
                        .map_err(|fallback_err| {
                            format!(
                                "Primary ({}) and fallback ({}) failed. Primary error: {} | Fallback error: {}",
                                self.model_name, fallback_model, primary_err, fallback_err
                            )
                        })
                } else {
                    Err(primary_err)
                }
            }
        }
    }

    async fn chat_stream_with_model(
        &self,
        base_url: &str,
        model_name: &str,
        messages: Vec<ChatMessage>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk, String>> + Send>>, String>
    {
        let url = format!("{}/chat/completions", base_url);

        let body = serde_json::json!({
            "model": model_name,
            "messages": messages,
            "stream": true,
            "temperature": self.generation_temperature(),
            "max_tokens": self.generation_max_tokens(),
            "top_p": self.generation_top_p(),
        });

        let mut req = self.client.post(&url).json(&body);

        if let Some(key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let resp = req.send().await.map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            log::error!("AI Stream Error: {}", err_text);
            return Err(format!("AI Stream Error ({}): {}", status, err_text));
        }

        let is_event_stream = resp
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_ascii_lowercase().contains("text/event-stream"))
            .unwrap_or(false);

        if !is_event_stream {
            let mut parsed = resp
                .json::<ChatCompletionResponse>()
                .await
                .map_err(|e| format!("AI Stream JSON fallback parse error: {}", e))?;
            Self::normalize_reasoning_content(&mut parsed);
            let chunks = Self::response_to_stream_chunks(parsed);
            return Ok(Box::pin(futures_util::stream::iter(chunks)));
        }

        let stream = resp.bytes_stream();
        let mut buffer = String::new();

        let transformed_stream = stream
            .map(
                move |chunk_res: Result<bytes::Bytes, reqwest::Error>| match chunk_res {
                    Ok(bytes) => {
                        let chunk_text = String::from_utf8_lossy(bytes.as_ref());
                        buffer.push_str(&chunk_text);

                        let mut results = Vec::new();
                        while let Some(line_end) = buffer.find('\n') {
                            let line = buffer[..line_end].trim().to_string();
                            buffer.drain(..=line_end);

                            if line.is_empty() {
                                continue;
                            }

                            if line.starts_with("data: ") {
                                let json_str = &line[6..];
                                if json_str == "[DONE]" {
                                    continue;
                                }

                                match serde_json::from_str::<ChatCompletionChunk>(json_str) {
                                    Ok(chunk) => {
                                        results.push(Ok(chunk));
                                    }
                                    Err(e) => {
                                        log::warn!(
                                            "Failed to parse stream chunk JSON: {}. JSON was: {}",
                                            e,
                                            json_str
                                        );
                                    }
                                }
                            } else {
                                log::debug!("Received non-data line from stream: {}", line);
                            }
                        }
                        results
                    }
                    Err(e) => {
                        log::error!("Stream connection error: {}", e);
                        vec![Err(e.to_string())]
                    }
                },
            )
            .flat_map(futures_util::stream::iter);

        Ok(Box::pin(transformed_stream))
    }

    pub async fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk, String>> + Send>>, String>
    {
        // Log messages for debugging (local only)
        if self.provider == "docker-model-runner" {
            log::info!("Streaming {} messages to local AI", messages.len());
            for (i, msg) in messages.iter().enumerate() {
                log::debug!(
                    "  [{}] {}: {} chars",
                    i,
                    serde_json::to_string(&msg.role).unwrap_or_default(),
                    msg.content.len()
                );
            }
        }

        let primary = self
            .chat_stream_with_model(&self.base_url, &self.model_name, messages.clone())
            .await;

        match primary {
            Ok(stream) => Ok(stream),
            Err(primary_err) => {
                if let (Some(fallback_url), Some(fallback_model)) =
                    (&self.fallback_base_url, &self.fallback_model_name)
                {
                    log::warn!(
                        "Primary stream model '{}' failed, trying fallback '{}': {}",
                        self.model_name,
                        fallback_model,
                        primary_err
                    );
                    self.chat_stream_with_model(fallback_url, fallback_model, messages)
                        .await
                        .map_err(|fallback_err| {
                            format!(
                                "Primary stream ({}) and fallback stream ({}) failed. Primary error: {} | Fallback error: {}",
                                self.model_name, fallback_model, primary_err, fallback_err
                            )
                        })
                } else {
                    Err(primary_err)
                }
            }
        }
    }
}
