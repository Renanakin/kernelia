# Integracion Tauri + SvelteKit con LLM real

## Endpoint objetivo
- Proxy OpenAI-compatible: `http://localhost:11435/v1/chat/completions`
- Backend real: Ollama en `http://localhost:11434`

## Frontend (SvelteKit)
```js
const r = await fetch('http://localhost:11435/v1/chat/completions', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    model: 'gemma3',
    messages: [{ role: 'user', content: 'Tu pregunta' }]
  })
});
const data = await r.json();
```

## Tauri (Rust)
```rust
#[tauri::command]
pub async fn chat_with_llm(msg: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "model": "gemma3",
        "messages": [{"role":"user","content": msg}]
    });

    let resp = client
        .post("http://localhost:11435/v1/chat/completions")
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let value: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let text = value["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(text)
}
```
