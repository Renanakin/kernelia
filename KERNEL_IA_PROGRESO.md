# KERNEL IA — Estado del Proyecto

> **Última actualización:** 8 de mayo de 2026 (Kernel IA v2.1.0)
> **Repo:** https://github.com/Renanakin/nexus-lite (privado)  
> **Branch activo:** `develop`  
> **Último commit:** `21fe64f`

---

## 🛡️ Sistema de Control de Acceso (RBAC) - NUEVO

KERNEL IA ahora implementa un robusto sistema de Control de Acceso Basado en Roles (RBAC) para proteger las herramientas críticas del sistema.

### Roles Disponibles:
- **Viewer (Observador):** Solo puede ver información básica (CPU, RAM, discos, archivos basura). No puede modificar el sistema.
- **PowerUser (Usuario Avanzado):** Puede ejecutar diagnósticos de red y analizar el registro, pero no puede reiniciar servicios críticos.
- **Owner (Propietario):** Acceso total a todas las herramientas del núcleo (gestión de servicios, limpieza profunda, etc.).

### Implementación:
- **Backend:** `src-tauri/src/tools/rbac.rs` define las políticas de ejecución.
- **Frontend:** `QuickChecks.svelte` oculta y bloquea herramientas según el rol activo.
- **Persistencia:** El rol se guarda en `settings.json` y se sincroniza automáticamente.

---

## ⚠️ IMPORTANTE: PowerShell en Modo Administrador

**TODA ejecución de PowerShell relacionada con este proyecto DEBE ser en modo Administrador.**

Esto aplica para:
- `pnpm tauri dev` (desarrollo)
- `npx tauri build` (compilación)
- Cualquier comando de `cargo`
- Git push/pull
- Ejecución del `.exe` final

**Razón:** KERNEL IA ejecuta comandos del sistema (PowerShell, CMD), accede a información de hardware (CPU, RAM, discos, GPU, procesos), lee/escribe archivos del sistema. Sin privilegios de administrador, muchas herramientas del asistente fallarán silenciosamente o darán información incompleta.

### Cómo abrir PowerShell como Administrador:
1. Click derecho en el ícono de PowerShell → "Ejecutar como administrador"
2. O desde VS Code: configurar el terminal integrado para abrir elevado
3. O crear un atajo con la propiedad "Ejecutar como administrador" marcada

### Configurar VS Code para terminal Admin:
```json
// settings.json de VS Code
{
  "terminal.integrated.profiles.windows": {
    "PowerShell (Admin)": {
      "source": "PowerShell",
      "args": ["-NoProfile"],
      "icon": "terminal-powershell"
    }
  }
}
```
> **Nota:** VS Code no puede elevar terminales automáticamente. Se recomienda ejecutar VS Code como administrador directamente.

---

## Stack Tecnológico

| Componente | Tecnología | Versión |
|---|---|---|
| Framework desktop | Tauri | 2.10.0 |
| Frontend | SvelteKit (Svelte 5) | adapter-static SPA |
| Estilos | Tailwind CSS | 4.x vía @tailwindcss/vite |
| Backend | Rust | 1.93.1 |
| Compilador C++ | MSVC (VS 2022 Community) | 19.44 x64 |
| Node.js | v22.19.0 | pnpm 10.30.3 |
| Encriptación | AES-256-GCM (ring) | — |
| HTTP | reqwest + rustls-tls | 0.12 |

---

## Modelos de IA Configurados

| Modelo | Proveedor | ID interno | Estado |
|---|---|---|---|
| Llama 3.3 70B | Groq | `llama-groq` | ✅ Funcional |
| DeepSeek V3 | DeepSeek | `deepseek-v3` | ✅ Funcional (mejor hasta ahora) |
| Qwen 2.5 72B | OpenRouter | `qwen-openrouter` | ✅ Configurado |
| GLM-3 Turbo | Zhipu AI | `glm-5` | ✅ Corregido (era glm-5, ahora glm-3-turbo) |
| Gemini 2.0 Flash | Google | `gemini-flash` | ⚠️ Sin adapter (API diferente) |

### API Keys
- Almacenadas en `.env` (gitignored, nunca se commitean)
- Se encriptan con AES-256-GCM al iniciar y se guardan en `settings.json`
- También se pueden configurar desde la UI (⚙️ Configuración)

---

## Historial de Commits

```
21fe64f fix: Groq null content parse, GLM-3-Turbo model name, KERNEL IA label
07bd399 feat: load API keys from .env (encrypted, never committed)
155a494 feat: rebrand to KERNEL IA identity
baadb83 feat: enable cloud models (Groq, DeepSeek, OpenRouter, Zhipu)
66f7de7 fix: resolve Rust compilation errors and Svelte 5 syntax issues
f94e3b2 feat: initial NEXUS LITE scaffold with Tauri 2 + SvelteKit + Tailwind CSS 4
```

---

## Bugs Corregidos

### 1. Groq: Error en segunda respuesta
- **Problema:** `Failed to parse API response: error decoding response body`
- **Causa:** Groq envía `content: null` cuando el modelo responde con tool_calls. Rust esperaba `String`.
- **Fix:** Custom deserializer `deserialize_content` que convierte `null` → `""` en `ChatMessage.content`
- **Archivo:** `src-tauri/src/ai/models.rs`

### 2. GLM: Modelo inexistente
- **Problema:** La API de Zhipu no reconocía `glm-5`
- **Causa:** El model_name correcto es `glm-3-turbo`
- **Fix:** Cambiado en `default_models()` de settings.rs
- **Archivo:** `src-tauri/src/config/settings.rs`

### 3. DeepSeek: Insufficient Balance (402)
- **Problema:** `402 Payment Required: Insufficient Balance`
- **Causa:** La cuenta de DeepSeek no tiene créditos suficientes
- **Fix:** No es bug de código — recargar saldo en platform.deepseek.com

---

## Estructura de Archivos Clave

```
nexus-lite/
├── .env                          # API keys (GITIGNORED)
├── .gitignore
├── package.json
├── src/
│   ├── app.css                   # Design system KERNEL IA
│   ├── app.html                  # HTML base
│   └── lib/
│       ├── components/
│       │   ├── ChatWindow.svelte      # Componente principal
│       │   ├── MessageBubble.svelte   # Burbujas de mensaje
│       │   ├── InputBar.svelte        # Barra de entrada
│       │   ├── ModelSelector.svelte   # Selector de modelo
│       │   ├── WelcomeScreen.svelte   # Pantalla de bienvenida
│       │   ├── SettingsModal.svelte   # Modal de configuración
│       │   └── ActionIndicator.svelte # Indicador de herramientas
│       ├── stores/
│       │   ├── chat.js
│       │   ├── settings.js
│       │   └── system.js
│       └── utils/
│           ├── markdown.js
│           └── formatting.js
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── lib.rs                     # Entry point + dotenv loader
│       ├── ai/
│       │   ├── models.rs              # Structs de la API
│       │   ├── router.rs              # Router + system prompt
│       │   ├── function_calling.rs    # Loop de function calling
│       │   └── mod.rs
│       ├── config/
│       │   ├── settings.rs            # Configuración + modelos
│       │   ├── encryption.rs          # AES-256-GCM
│       │   └── mod.rs
│       ├── commands/
│       │   ├── chat.rs                # Comandos Tauri (chat)
│       │   ├── system.rs              # Comandos Tauri (sistema)
│       │   └── mod.rs
│       └── tools/
│           ├── terminal.rs            # Ejecución de comandos
│           ├── filesystem.rs          # Lectura/escritura archivos
│           ├── sysinfo_tool.rs        # Info del sistema
│           ├── processes.rs           # Gestión de procesos
│           └── mod.rs
```

---

## Pendientes

- [ ] **PowerShell Admin:** Configurar que los comandos internos de la app se ejecuten con privilegios elevados
- [ ] **Gemini adapter:** Implementar adapter para la API de Google (formato diferente a OpenAI)
- [ ] **Streaming:** Implementar respuestas en streaming (SSE) para mejor UX
- [ ] **Ollama local:** Probar con modelos locales vía Ollama
- [ ] **Auto-update:** Sistema de actualización automática
- [ ] **Logging visual:** Mostrar logs del backend en la UI para debugging
- [ ] **Tests:** Tests unitarios para tools y function calling

---

## Comando para Ejecutar (siempre como Admin)

```powershell
# 1. Abrir PowerShell como Administrador
# 2. Navegar al proyecto
cd K:\desarrollos\proyectoconmcp\nexus-lite

# 3. Cargar entorno MSVC
$env:Path = [System.Environment]::GetEnvironmentVariable("Path", "User") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "Machine")
cmd /c "`"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat`" x64 && set" | ForEach-Object { if ($_ -match "^(.*?)=(.*)$") { [System.Environment]::SetEnvironmentVariable($matches[1], $matches[2], "Process") } }

# 4. Ejecutar en modo desarrollo
pnpm tauri dev

# 5. Para compilar el .exe final
npx tauri build
```
