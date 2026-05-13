# Integración de LLM Locales y Externos - Nexus-Lite (KernelIA)

## 1. Estrategia de Integración
Nexus-Lite utiliza una arquitectura agnóstica al proveedor mediante el estándar de API de OpenAI. Esto permite alternar entre modelos locales de alto rendimiento y modelos en la nube de gran capacidad.

## 2. Proveedores Soportados

### Locales (Privacidad Total)
- **Ollama**: Servidor local que corre en el puerto `11434`.
- **LM Studio**: Servidor local compatible con OpenAI en el puerto `1234`.

### Externos (Alta Capacidad)
- **OpenRouter**: Agregador de modelos (Claude, GPT-4, Llama 3).
- **DeepSeek**: Modelos optimizados para razonamiento.
- **Groq**: Inferencia de ultra-alta velocidad.

## 3. Configuración del Endpoint Genérico
El sistema inyecta dinámicamente la `base_url` y `api_key` configuradas en `settings.rs`.

```rust
let url = format!("{}/chat/completions", model_config.base_url);
let mut req_builder = client.post(&url);

// Detección automática de headers
if model_config.is_local {
    // Ollama/LM Studio no suelen requerir API Key, pero se envía si existe
    req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
} else {
    // Configuración específica para OpenRouter, etc.
}
```

## 4. Function Calling (Herramientas)
El loop de orquestación en `function_calling.rs` maneja el envío de definiciones de herramientas (specs) al modelo. Cuando el modelo responde con una solicitud de ejecución de herramienta, Nexus-Lite:
1. Valida permisos (RBAC).
2. Ejecuta la herramienta en el núcleo de Rust.
3. Devuelve el resultado al modelo para que genere la respuesta final al usuario.

## 5. Optimización de Latencia
Para modelos locales con hardware limitado, se recomienda:
- Usar modelos de parámetros reducidos (e.g., Llama 3 8B, Phi-3).
- Implementar **Quick Checks** para evitar llamadas innecesarias al LLM.
