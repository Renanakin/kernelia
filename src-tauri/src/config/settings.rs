use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// Quick Checks: DiagnÃ³sticos rÃ¡pidos pre-configurados
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QuickCheckType {
    DirectTool,
    LlmPrompt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickCheck {
    pub id: String,
    pub label: String,
    pub description: String,
    pub icon: String,
    pub color: String,
    #[serde(rename = "type")]
    pub kind: QuickCheckType,
    pub required_permissions: Vec<String>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub tool_pipeline: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickCheckResult {
    pub id: String,
    pub label: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SupportProfile {
    Superusuario,
    Soporte1,
    Tecnico,
}

impl SupportProfile {
    pub fn to_rbac_role(&self) -> crate::tools::rbac::UserRole {
        match self {
            SupportProfile::Superusuario => crate::tools::rbac::UserRole::Owner,
            SupportProfile::Soporte1 => crate::tools::rbac::UserRole::PowerUser,
            SupportProfile::Tecnico => crate::tools::rbac::UserRole::Viewer,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccount {
    pub username: String,
    pub password_encrypted: String,
    pub profile: SupportProfile,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthState {
    pub is_authenticated: bool,
    pub current_username: Option<String>,
    pub current_profile: Option<SupportProfile>,
    pub tecnico_critical_unlock_until_epoch: Option<i64>,
}

impl Default for AuthState {
    fn default() -> Self {
        Self {
            is_authenticated: false,
            current_username: None,
            current_profile: None,
            tecnico_critical_unlock_until_epoch: None,
        }
    }
}

/// Modelo de IA con su configuraciÃ³n de conexiÃ³n
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub model_name: String,
    pub supports_function_calling: bool,
    pub is_local: bool,
    pub max_tokens: u32,
}

/// ConfiguraciÃ³n de Ollama (local o en red)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub model: String,
    pub network_hosts: Vec<String>,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "localhost".into(),
            port: 11434,
            model: "qwen2.5:3b".into(),
            network_hosts: vec!["192.168.1.101:11434".into(), "192.168.1.102:11434".into()],
        }
    }
}

/// ConfiguraciÃ³n general de la aplicaciÃ³n
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub version: String,
    pub first_run: bool,
    pub selected_model: String,
    #[serde(default = "AppSettings::default_rag_engine_enabled")]
    pub rag_engine_enabled: bool,
    #[serde(default = "AppSettings::default_rag_compare_mode")]
    pub rag_compare_mode: bool,
    #[serde(default = "AppSettings::default_rag_debug_panel")]
    pub rag_debug_panel: bool,
    #[serde(default = "AppSettings::default_rag_show_confidence_badge")]
    pub rag_show_confidence_badge: bool,
    pub models: Vec<ModelConfig>,
    pub ollama: OllamaConfig,
    pub language: String,
    pub theme: String,
    pub max_history_messages: usize,
    pub confirm_dangerous_actions: bool,
    pub user_role: crate::tools::rbac::UserRole,
    #[serde(default = "AppSettings::default_users")]
    pub users: Vec<UserAccount>,
    #[serde(default)]
    pub auth: AuthState,
    pub tecnico_critical_password_encrypted: Option<String>,
    pub megaboss_password_encrypted: Option<String>,
    pub megaboss_unlock_until_epoch: Option<i64>,
    #[serde(default = "AppSettings::default_quick_checks")]
    pub quick_checks: Vec<QuickCheck>,
}

impl Default for AppSettings {
    fn default() -> Self {
        let tecnico_pwd = super::encryption::Encryption::encrypt("KernelIA!CriticalProc2026").ok();
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            first_run: true,
            selected_model: "gemma3-local".to_string(),
            rag_engine_enabled: Self::default_rag_engine_enabled(),
            rag_compare_mode: Self::default_rag_compare_mode(),
            rag_debug_panel: Self::default_rag_debug_panel(),
            rag_show_confidence_badge: Self::default_rag_show_confidence_badge(),
            models: Self::default_models(),
            ollama: OllamaConfig::default(),
            language: "es".to_string(),
            theme: "dark".to_string(),
            max_history_messages: 50,
            confirm_dangerous_actions: true,
            user_role: crate::tools::rbac::UserRole::Viewer,
            users: Self::default_users(),
            auth: AuthState::default(),
            tecnico_critical_password_encrypted: tecnico_pwd,
            megaboss_password_encrypted: None,
            megaboss_unlock_until_epoch: None,
            quick_checks: Self::default_quick_checks(),
        }
    }
}

impl AppSettings {
    fn default_rag_engine_enabled() -> bool {
        std::env::var("KERNELIA_RAG_ENABLED")
            .ok()
            .map(|value| matches!(value.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or(true)
    }

    fn default_rag_compare_mode() -> bool {
        std::env::var("KERNELIA_RAG_COMPARE_MODE")
            .ok()
            .map(|value| matches!(value.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or(false)
    }

    fn default_rag_debug_panel() -> bool {
        std::env::var("KERNELIA_RAG_DEBUG_PANEL")
            .ok()
            .map(|value| matches!(value.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or(false)
    }

    fn default_rag_show_confidence_badge() -> bool {
        true
    }

    fn default_gemma3_base_url() -> String {
        std::env::var("KERNELIA_GEMMA3_BASE_URL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "http://localhost:11435/v1".to_string())
    }

    fn enforce_local_docker_models(&mut self) -> bool {
        let before = self.models.len();
        self.models.retain(|m| m.id != "gemma4-local");
        let mut changed = self.models.len() != before;

        if !self.models.iter().any(|m| m.id == "gemma3-local") {
            self.models.insert(
                0,
                ModelConfig {
                    id: "gemma3-local".into(),
                    name: "Gemma 3 Local (Docker)".into(),
                    provider: "docker-model-runner".into(),
                    base_url: Self::default_gemma3_base_url(),
                    api_key: None,
                    model_name: "gemma:2b".into(),
                    supports_function_calling: true,
                    is_local: true,
                    max_tokens: 500,
                },
            );
            changed = true;
        }

        if self.selected_model != "gemma3-local" {
            self.selected_model = "gemma3-local".to_string();
            changed = true;
        }

        changed
    }

    fn refresh_default_quick_checks(&mut self) -> bool {
        let defaults = Self::default_quick_checks();
        let mut changed = false;

        for default_check in defaults {
            match self
                .quick_checks
                .iter_mut()
                .find(|q| q.id == default_check.id)
            {
                Some(existing) => {
                    if existing.label != default_check.label
                        || existing.description != default_check.description
                        || existing.icon != default_check.icon
                        || existing.color != default_check.color
                        || existing.kind != default_check.kind
                        || existing.required_permissions != default_check.required_permissions
                        || existing.prompt != default_check.prompt
                        || existing.tool_pipeline != default_check.tool_pipeline
                    {
                        *existing = default_check;
                        changed = true;
                    }
                }
                None => {
                    self.quick_checks.push(default_check);
                    changed = true;
                }
            }
        }

        changed
    }

    fn migrate_legacy_model_runner_urls(&mut self) -> bool {
        let mut changed = false;
        for model in &mut self.models {
            if model.base_url.contains("localhost:21434")
                || model.base_url.contains("localhost:11435")
            {
                model.base_url = model.base_url.replace("localhost:21434", "localhost:11435");
                changed = true;
            }
            if model.provider == "docker-model-runner"
                && model.base_url.contains("/engines/llama.cpp/v1")
            {
                model.base_url = model.base_url.replace("/engines/llama.cpp/v1", "/v1");
                changed = true;
            }
            if model.id == "gemma3-local" && model.model_name != "gemma:2b" {
                model.model_name = "gemma:2b".to_string();
                changed = true;
            }
        }
        changed
    }

    /// Obtiene la ruta del archivo de configuraciÃ³n (portable: junto al .exe)
    pub fn config_path() -> PathBuf {
        // Intenta ruta portable (junto al ejecutable)
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let portable_config = exe_dir.join("config").join("settings.json");
                if portable_config.parent().map_or(false, |p| p.exists())
                    || portable_config.exists()
                {
                    return portable_config;
                }
            }
        }

        // Fallback: directorio de datos del usuario
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("nexus-lite");
        config_dir.join("settings.json")
    }

    /// Carga la configuraciÃ³n desde archivo o crea una por defecto
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str::<AppSettings>(&content) {
                        Ok(mut settings) => {
                            let mut changed = false;
                            if settings.refresh_default_quick_checks() {
                                changed = true;
                            }
                            if settings.users.is_empty() {
                                settings.users = Self::default_users();
                                changed = true;
                            }
                            if settings.tecnico_critical_password_encrypted.is_none() {
                                settings.tecnico_critical_password_encrypted =
                                    super::encryption::Encryption::encrypt(
                                        "KernelIA!CriticalProc2026",
                                    )
                                    .ok();
                                changed = true;
                            }
                            if settings.migrate_legacy_model_runner_urls() {
                                changed = true;
                            }
                            if settings.enforce_local_docker_models() {
                                changed = true;
                            }
                            // Siempre inicia bloqueado hasta login.
                            if settings.auth.is_authenticated
                                || settings.auth.current_username.is_some()
                                || settings.auth.current_profile.is_some()
                                || settings.auth.tecnico_critical_unlock_until_epoch.is_some()
                            {
                                settings.auth = AuthState::default();
                                changed = true;
                            }
                            if settings.user_role != crate::tools::rbac::UserRole::Viewer {
                                settings.user_role = crate::tools::rbac::UserRole::Viewer;
                                changed = true;
                            }
                            if changed {
                                let _ = settings.save();
                            }
                            return settings;
                        }
                        Err(e) => {
                            log::warn!("Error parsing settings, using defaults: {}", e);
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Error reading settings file, using defaults: {}", e);
                }
            }
        }

        let settings = AppSettings::default();
        let _ = settings.save();
        settings
    }

    /// Guarda la configuraciÃ³n a disco
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Error creating config dir: {}", e))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Error serializing settings: {}", e))?;
        fs::write(&path, json).map_err(|e| format!("Error writing settings: {}", e))?;
        Ok(())
    }

    /// Obtiene la configuraciÃ³n del modelo actualmente seleccionado
    pub fn current_model(&self) -> Option<&ModelConfig> {
        self.models.iter().find(|m| m.id == self.selected_model)
    }

    /// Establece la API key para un modelo (encriptada)
    pub fn set_api_key(&mut self, model_id: &str, api_key: &str) -> Result<(), String> {
        if let Some(model) = self.models.iter_mut().find(|m| m.id == model_id) {
            let encrypted = super::encryption::Encryption::encrypt(api_key)?;
            model.api_key = Some(encrypted);
            self.save()?;
            Ok(())
        } else {
            Err(format!("Model '{}' not found", model_id))
        }
    }

    /// Obtiene la API key desencriptada de un modelo
    pub fn get_api_key(&self, model_id: &str) -> Option<String> {
        self.models
            .iter()
            .find(|m| m.id == model_id)
            .and_then(|m| m.api_key.as_ref())
            .and_then(|encrypted| super::encryption::Encryption::decrypt(encrypted).ok())
    }

    pub fn set_megaboss_password(&mut self, password: &str) -> Result<(), String> {
        if password.trim().len() < 6 {
            return Err("La clave MegaBoss debe tener al menos 6 caracteres.".to_string());
        }
        let encrypted = super::encryption::Encryption::encrypt(password)?;
        self.megaboss_password_encrypted = Some(encrypted);
        self.megaboss_unlock_until_epoch = None;
        self.save()
    }

    pub fn verify_megaboss_password(&self, password: &str) -> bool {
        match &self.megaboss_password_encrypted {
            Some(enc) => super::encryption::Encryption::decrypt(enc)
                .map(|plain| plain == password)
                .unwrap_or(false),
            None => false,
        }
    }

    pub fn unlock_megaboss_for_minutes(&mut self, minutes: i64) -> Result<(), String> {
        let now = chrono::Utc::now().timestamp();
        self.megaboss_unlock_until_epoch = Some(now + (minutes * 60));
        self.save()
    }

    pub fn is_megaboss_unlocked(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.megaboss_unlock_until_epoch
            .map(|until| until > now)
            .unwrap_or(false)
    }

    pub fn default_users() -> Vec<UserAccount> {
        let super_pass =
            super::encryption::Encryption::encrypt("KernelIA!Super2026").unwrap_or_default();
        let soporte_pass =
            super::encryption::Encryption::encrypt("KernelIA!Support2026").unwrap_or_default();
        let tecnico_pass =
            super::encryption::Encryption::encrypt("KernelIA!Tech2026").unwrap_or_default();

        vec![
            UserAccount {
                username: "superadmin".into(),
                password_encrypted: super_pass,
                profile: SupportProfile::Superusuario,
                active: true,
            },
            UserAccount {
                username: "soporte1".into(),
                password_encrypted: soporte_pass,
                profile: SupportProfile::Soporte1,
                active: true,
            },
            UserAccount {
                username: "tecnico".into(),
                password_encrypted: tecnico_pass,
                profile: SupportProfile::Tecnico,
                active: true,
            },
        ]
    }

    pub fn is_superuser(&self) -> bool {
        self.auth.current_profile == Some(SupportProfile::Superusuario)
    }

    pub fn login(&mut self, username: &str, password: &str) -> Result<SupportProfile, String> {
        let user = self
            .users
            .iter()
            .find(|u| u.active && u.username.eq_ignore_ascii_case(username))
            .ok_or_else(|| "Usuario no encontrado o inactivo".to_string())?
            .clone();

        let plain = super::encryption::Encryption::decrypt(&user.password_encrypted)
            .map_err(|_| "Credenciales invalidas".to_string())?;

        if plain != password {
            return Err("Credenciales invalidas".to_string());
        }

        self.auth.is_authenticated = true;
        self.auth.current_username = Some(user.username.clone());
        self.auth.current_profile = Some(user.profile.clone());
        self.auth.tecnico_critical_unlock_until_epoch = None;
        self.user_role = user.profile.to_rbac_role();
        self.save()?;
        Ok(user.profile)
    }

    pub fn logout(&mut self) -> Result<(), String> {
        self.auth = AuthState::default();
        self.user_role = crate::tools::rbac::UserRole::Viewer;
        self.save()
    }

    pub fn verify_tecnico_critical_password(&self, password: &str) -> bool {
        match &self.tecnico_critical_password_encrypted {
            Some(enc) => super::encryption::Encryption::decrypt(enc)
                .map(|plain| plain == password)
                .unwrap_or(false),
            None => false,
        }
    }

    pub fn unlock_tecnico_critical_for_minutes(&mut self, minutes: i64) -> Result<(), String> {
        if self.auth.current_profile != Some(SupportProfile::Tecnico) {
            return Err("Solo el perfil Tecnico puede usar esta elevacion".to_string());
        }
        let now = chrono::Utc::now().timestamp();
        self.auth.tecnico_critical_unlock_until_epoch = Some(now + (minutes * 60));
        self.user_role = crate::tools::rbac::UserRole::PowerUser;
        self.save()
    }

    pub fn refresh_tecnico_role_if_needed(&mut self) {
        if self.auth.current_profile == Some(SupportProfile::Tecnico) {
            let now = chrono::Utc::now().timestamp();
            let unlocked = self
                .auth
                .tecnico_critical_unlock_until_epoch
                .map(|until| until > now)
                .unwrap_or(false);
            self.user_role = if unlocked {
                crate::tools::rbac::UserRole::PowerUser
            } else {
                crate::tools::rbac::UserRole::Viewer
            };
        }
    }

    pub fn create_user(
        &mut self,
        username: &str,
        password: &str,
        profile: SupportProfile,
    ) -> Result<(), String> {
        if !self.is_superuser() {
            return Err("Solo el superusuario puede crear usuarios".to_string());
        }
        if username.trim().len() < 3 {
            return Err("El usuario debe tener al menos 3 caracteres".to_string());
        }
        if password.trim().len() < 8 {
            return Err("La contrasena debe tener al menos 8 caracteres".to_string());
        }
        if self
            .users
            .iter()
            .any(|u| u.username.eq_ignore_ascii_case(username))
        {
            return Err("El usuario ya existe".to_string());
        }

        let encrypted = super::encryption::Encryption::encrypt(password)?;
        self.users.push(UserAccount {
            username: username.trim().to_string(),
            password_encrypted: encrypted,
            profile,
            active: true,
        });
        self.save()
    }

    pub fn delete_user(&mut self, username: &str) -> Result<(), String> {
        if !self.is_superuser() {
            return Err("Solo el superusuario puede borrar usuarios".to_string());
        }

        let uname = username.trim();
        if uname.eq_ignore_ascii_case("superadmin") {
            return Err("No se puede borrar el superadmin base".to_string());
        }

        let before = self.users.len();
        self.users
            .retain(|u| !u.username.eq_ignore_ascii_case(uname));

        if self.users.len() == before {
            return Err("Usuario no encontrado".to_string());
        }
        self.save()
    }

    /// Inyecta API keys desde variables de entorno para modelos que no tienen key guardada.
    /// TambiÃ©n auto-migra modelos locales nuevos que no existan en configs guardadas.
    pub fn inject_env_keys(&mut self) {
        // Forzar modo estable con un solo modelo local (Gemma3).
        self.models.retain(|m| m.id != "gemma4-local");

        if let Some(g3) = self.models.iter_mut().find(|m| m.id == "gemma3-local") {
            g3.provider = "docker-model-runner".into();
            if let Ok(base_url) = std::env::var("KERNELIA_GEMMA3_BASE_URL") {
                if !base_url.trim().is_empty() {
                    g3.base_url = base_url;
                }
            }
            if g3.base_url.trim().is_empty() {
                g3.base_url = Self::default_gemma3_base_url();
            }
            g3.model_name = "gemma:2b".into();
            g3.name = "Gemma 3 Local (Docker)".into();
            g3.is_local = true;
            g3.supports_function_calling = true;
            g3.max_tokens = 500;
        } else {
            self.models.insert(
                0,
                ModelConfig {
                    id: "gemma3-local".into(),
                    name: "Gemma 3 Local (Docker)".into(),
                    provider: "docker-model-runner".into(),
                    base_url: Self::default_gemma3_base_url(),
                    api_key: None,
                model_name: "gemma:2b".into(),
                    supports_function_calling: true,
                    is_local: true,
                    max_tokens: 500,
                },
            );
            log::info!("Auto-migrated: Gemma 3 local model added to config");
        }

        let env_map: Vec<(&str, &str)> = vec![
            ("llama-groq", "GROQ_API_KEY"),
            ("deepseek-v3", "DEEPSEEK_API_KEY"),
            ("qwen-openrouter", "OPENROUTER_API_KEY"),
            ("glm-5", "GLM_API_KEY"),
            ("gemini-flash", "GEMINI_API_KEY"),
        ];

        for (model_id, env_var) in &env_map {
            if let Ok(key) = std::env::var(env_var) {
                if !key.is_empty() {
                    // Solo inyectar si el modelo no tiene key guardada
                    if let Some(model) = self.models.iter_mut().find(|m| m.id == *model_id) {
                        if model.api_key.is_none() {
                            // Encriptar y guardar para que persista
                            match super::encryption::Encryption::encrypt(&key) {
                                Ok(encrypted) => {
                                    model.api_key = Some(encrypted);
                                    log::info!("API key loaded from env for model: {}", model_id);
                                }
                                Err(e) => {
                                    log::warn!("Failed to encrypt env key for {}: {}", model_id, e);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Guardar las keys encriptadas a disco
        if let Err(e) = self.save() {
            log::warn!("Failed to save settings after env key injection: {}", e);
        }
    }

    /// Quick Checks por defecto (diagnÃ³sticos pre-configurados)
    pub fn default_quick_checks() -> Vec<QuickCheck> {
        vec![
            QuickCheck {
                id: "system_health".into(),
                label: "Salud del Sistema".into(),
                description: "CPU, RAM, Disco y temperatura en tiempo real.".into(),
                icon: "SYS".into(),
                color: "blue".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["get_system_info".into()],
                prompt: None,
                tool_pipeline: vec!["get_system_info".into()],
            },
            QuickCheck {
                id: "slow_pc".into(),
                label: "Por que esta lenta?".into(),
                description: "Analisis IA de CPU/RAM/procesos con diagnostico ejecutivo.".into(),
                icon: "SLOW".into(),
                color: "orange".into(),
                kind: QuickCheckType::LlmPrompt,
                required_permissions: vec!["get_system_info".into(), "list_processes".into()],
                prompt: Some(
                    "Analiza el rendimiento actual del sistema. Revisa CPU, RAM y los procesos \
                     que mas consumen recursos. Proporciona un diagnostico conciso en espanol con \
                     3 causas probables y 3 acciones concretas para mejorar el rendimiento. \
                     No uses emojis. Entrega respuesta clara para usuario final."
                    .into(),
                ),
                tool_pipeline: vec![],
            },
            QuickCheck {
                id: "junk_scan".into(),
                label: "Escaneo de Basura".into(),
                description: "Detecta archivos temporales y espacio recuperable.".into(),
                icon: "JUNK".into(),
                color: "yellow".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["analyze_junk".into()],
                prompt: None,
                tool_pipeline: vec!["analyze_junk".into()],
            },
            QuickCheck {
                id: "network_check".into(),
                label: "Chequeo de Red".into(),
                description: "Ping, DNS y latencia hacia servidores clave.".into(),
                icon: "NET".into(),
                color: "green".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["run_network_diagnostic".into()],
                prompt: None,
                tool_pipeline: vec!["run_network_diagnostic".into()],
            },
            QuickCheck {
                id: "autostart_audit".into(),
                label: "Auditoria de Inicio".into(),
                description: "Apps que inician con Windows y usan recursos.".into(),
                icon: "BOOT".into(),
                color: "purple".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["analyze_registry".into()],
                prompt: None,
                tool_pipeline: vec!["analyze_registry".into()],
            },
            QuickCheck {
                id: "health_score".into(),
                label: "Health Score".into(),
                description: "Riesgo operacional, anomalias y tendencia del equipo.".into(),
                icon: "HEALTH".into(),
                color: "green".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["health_overview".into()],
                prompt: None,
                tool_pipeline: vec!["health_overview".into()],
            },
            QuickCheck {
                id: "automation_plan".into(),
                label: "Automatizacion Inteligente".into(),
                description: "Evalua reglas SI/ENTONCES y plan de remediacion.".into(),
                icon: "AUTO".into(),
                color: "blue".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["run_automation_cycle".into()],
                prompt: None,
                tool_pipeline: vec!["run_automation_cycle".into()],
            },
            QuickCheck {
                id: "asset_inventory".into(),
                label: "Inventario Automatico".into(),
                description: "Escaneo de hardware/software del endpoint.".into(),
                icon: "INV".into(),
                color: "yellow".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["scan_asset_inventory".into()],
                prompt: None,
                tool_pipeline: vec!["scan_asset_inventory".into()],
            },
            QuickCheck {
                id: "incident_tickets".into(),
                label: "Tickets IA".into(),
                description: "Consulta incidentes creados automaticamente.".into(),
                icon: "TICKET".into(),
                color: "orange".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["list_incident_tickets".into()],
                prompt: None,
                tool_pipeline: vec!["list_incident_tickets".into()],
            },
            QuickCheck {
                id: "ops_docs".into(),
                label: "Documentacion IA".into(),
                description: "Genera runbook tecnico con evidencia de operaciones.".into(),
                icon: "DOC".into(),
                color: "red".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["generate_operational_documentation".into()],
                prompt: None,
                tool_pipeline: vec!["generate_operational_documentation".into()],
            },
            QuickCheck {
                id: "multi_tenant".into(),
                label: "Multiempresa".into(),
                description: "Vista consolidada de tenants, endpoints y salud operacional.".into(),
                icon: "TENANT".into(),
                color: "purple".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["cloud_multi_tenant_overview".into()],
                prompt: None,
                tool_pipeline: vec!["cloud_multi_tenant_overview".into()],
            },
            QuickCheck {
                id: "multiagent".into(),
                label: "Multiagente".into(),
                description: "Diagnostico coordinado por agentes especialistas.".into(),
                icon: "AGENT".into(),
                color: "green".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["run_multiagent_diagnosis".into()],
                prompt: None,
                tool_pipeline: vec!["run_multiagent_diagnosis".into()],
            },
            QuickCheck {
                id: "remote_support".into(),
                label: "Soporte Remoto".into(),
                description: "Sesiones remotas activas/cerradas con trazabilidad.".into(),
                icon: "REMOTE".into(),
                color: "blue".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["list_remote_support_sessions".into()],
                prompt: None,
                tool_pipeline: vec!["list_remote_support_sessions".into()],
            },
            QuickCheck {
                id: "rollback_ops".into(),
                label: "Rollback".into(),
                description: "Snapshots operativos disponibles para reversión segura.".into(),
                icon: "ROLL".into(),
                color: "orange".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["list_rollback_snapshots".into()],
                prompt: None,
                tool_pipeline: vec!["list_rollback_snapshots".into()],
            },
            QuickCheck {
                id: "trusted_exec".into(),
                label: "Trusted Exec".into(),
                description: "Verifica integridad del binario contra attestations previas.".into(),
                icon: "TRUST".into(),
                color: "yellow".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["verify_release_attestation".into()],
                prompt: None,
                tool_pipeline: vec!["verify_release_attestation".into()],
            },
            QuickCheck {
                id: "phase3_smoke".into(),
                label: "Fase 3 Smoke".into(),
                description: "Validación E2E de multiempresa, multiagente, remoto, rollback y trusted exec.".into(),
                icon: "SMOKE".into(),
                color: "red".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["run_phase3_smoke".into()],
                prompt: None,
                tool_pipeline: vec!["run_phase3_smoke".into()],
            },
            QuickCheck {
                id: "proactive_maintenance".into(),
                label: "Mantenimiento Proactivo".into(),
                description: "Ciclo preventivo con salud, multiagente y alerta operativa.".into(),
                icon: "PRO".into(),
                color: "green".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["run_proactive_maintenance".into()],
                prompt: None,
                tool_pipeline: vec!["run_proactive_maintenance".into()],
            },
            QuickCheck {
                id: "proactive_alerts".into(),
                label: "Alertas Proactivas".into(),
                description: "Historial de alertas preventivas y acciones sugeridas.".into(),
                icon: "ALERT".into(),
                color: "orange".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["list_proactive_alerts".into()],
                prompt: None,
                tool_pipeline: vec!["list_proactive_alerts".into()],
            },
            QuickCheck {
                id: "multimodel_route".into(),
                label: "Ruta Multimodelo".into(),
                description: "Recomendación dinámica del modelo IA según contexto de tarea.".into(),
                icon: "ROUTE".into(),
                color: "purple".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["recommend_model_route".into()],
                prompt: None,
                tool_pipeline: vec!["recommend_model_route".into()],
            },
            QuickCheck {
                id: "phase4_smoke".into(),
                label: "Fase 4 Smoke".into(),
                description: "Validación E2E de proactividad, scheduler y multimodelo.".into(),
                icon: "F4".into(),
                color: "red".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["run_phase4_smoke".into()],
                prompt: None,
                tool_pipeline: vec!["run_phase4_smoke".into()],
            },
            QuickCheck {
                id: "cloud_sync".into(),
                label: "Cloud Sync".into(),
                description: "Sincroniza diagnóstico con Hackteck Cloud y genera ticket.".into(),
                icon: "CLOUD".into(),
                color: "blue".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["upload_cloud_report".into()],
                prompt: None,
                tool_pipeline: vec!["upload_cloud_report".into()],
            },
            QuickCheck {
                id: "enterprise_dashboard".into(),
                label: "Enterprise Dashboard".into(),
                description: "KPIs de operación cloud, tickets, alertas y casos.".into(),
                icon: "ENT".into(),
                color: "purple".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["get_enterprise_dashboard".into()],
                prompt: None,
                tool_pipeline: vec!["get_enterprise_dashboard".into()],
            },
            QuickCheck {
                id: "enterprise_cases".into(),
                label: "Casos Enterprise".into(),
                description: "Consulta casos de soporte escalados en cloud.".into(),
                icon: "CASE".into(),
                color: "orange".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["list_support_cases".into()],
                prompt: None,
                tool_pipeline: vec!["list_support_cases".into()],
            },
            QuickCheck {
                id: "phase5_smoke".into(),
                label: "Fase 5 Smoke".into(),
                description: "Validación E2E de sincronización cloud y dashboard enterprise.".into(),
                icon: "F5".into(),
                color: "red".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["run_phase5_smoke".into()],
                prompt: None,
                tool_pipeline: vec!["run_phase5_smoke".into()],
            },
            QuickCheck {
                id: "kernel_slowpc".into(),
                label: "Kernel Slow PC".into(),
                description: "Diagnóstico de PC lenta con causas probables y acciones.".into(),
                icon: "KPC".into(),
                color: "orange".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["run_kernel_slowpc_diagnostic".into()],
                prompt: None,
                tool_pipeline: vec!["run_kernel_slowpc_diagnostic".into()],
            },
            QuickCheck {
                id: "kernel_network".into(),
                label: "Kernel Network".into(),
                description: "Playbook guiado para conectividad y latencia de red.".into(),
                icon: "KNET".into(),
                color: "blue".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["run_kernel_network_playbook".into()],
                prompt: None,
                tool_pipeline: vec!["run_kernel_network_playbook".into()],
            },
            QuickCheck {
                id: "kernel_guardrails".into(),
                label: "Kernel Guardrails".into(),
                description: "Verificación activa de bloqueo para comandos destructivos.".into(),
                icon: "SAFE".into(),
                color: "red".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["validate_kernel_guardrails".into()],
                prompt: None,
                tool_pipeline: vec!["validate_kernel_guardrails".into()],
            },
            QuickCheck {
                id: "kernel_autonomous".into(),
                label: "Kernel Autónomo".into(),
                description: "Workflow IF/THEN con simulación o ejecución controlada y verificación posterior.".into(),
                icon: "AUTO".into(),
                color: "orange".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["run_kernel_autonomous_workflow".into()],
                prompt: None,
                tool_pipeline: vec!["run_kernel_autonomous_workflow".into()],
            },
            QuickCheck {
                id: "kernel_automation_runs".into(),
                label: "Kernel Runs".into(),
                description: "Histórico de ejecuciones autónomas de la fase 6.".into(),
                icon: "KRUN".into(),
                color: "blue".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["list_kernel_automation_runs".into()],
                prompt: None,
                tool_pipeline: vec!["list_kernel_automation_runs".into()],
            },
            QuickCheck {
                id: "kernel_readiness".into(),
                label: "Kernel Readiness".into(),
                description: "Reporte de preparación operacional de diagnósticos y guardrails.".into(),
                icon: "KRDY".into(),
                color: "purple".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["generate_kernelia_readiness_report".into()],
                prompt: None,
                tool_pipeline: vec!["generate_kernelia_readiness_report".into()],
            },
            QuickCheck {
                id: "phase6_smoke".into(),
                label: "Fase 6 Smoke".into(),
                description: "Validación E2E de diagnósticos KernelIA y guardrails.".into(),
                icon: "F6".into(),
                color: "red".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["run_phase6_smoke".into()],
                prompt: None,
                tool_pipeline: vec!["run_phase6_smoke".into()],
            },
            QuickCheck {
                id: "latency_probe".into(),
                label: "Latency Probe".into(),
                description: "Mide latencia base del núcleo para monitoreo de respuesta.".into(),
                icon: "LAT".into(),
                color: "blue".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["run_latency_probe".into()],
                prompt: None,
                tool_pipeline: vec!["run_latency_probe".into()],
            },
            QuickCheck {
                id: "tool_benchmark".into(),
                label: "Tool Benchmark".into(),
                description: "Benchmark de herramientas críticas con métricas avg/p95.".into(),
                icon: "BENCH".into(),
                color: "orange".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["run_tool_benchmark".into()],
                prompt: None,
                tool_pipeline: vec!["run_tool_benchmark".into()],
            },
            QuickCheck {
                id: "performance_kpis".into(),
                label: "Performance KPIs".into(),
                description: "Consolidado de latencia y éxito operacional del sistema.".into(),
                icon: "KPI".into(),
                color: "purple".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["get_performance_kpis".into()],
                prompt: None,
                tool_pipeline: vec!["get_performance_kpis".into()],
            },
            QuickCheck {
                id: "noc_global".into(),
                label: "NOC Global".into(),
                description: "Estado NOC global multiempresa con SLA base y riesgo operativo.".into(),
                icon: "NOC".into(),
                color: "purple".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["get_noc_global_status".into()],
                prompt: None,
                tool_pipeline: vec!["get_noc_global_status".into()],
            },
            QuickCheck {
                id: "saas_licenses".into(),
                label: "SaaS Licenses".into(),
                description: "Inventario de licencias SaaS por tenant.".into(),
                icon: "LIC".into(),
                color: "blue".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["list_saas_licenses".into()],
                prompt: None,
                tool_pipeline: vec!["list_saas_licenses".into()],
            },
            QuickCheck {
                id: "noc_report".into(),
                label: "NOC Report".into(),
                description: "Genera reporte enterprise NOC consolidado.".into(),
                icon: "RPT".into(),
                color: "orange".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["generate_enterprise_noc_report".into()],
                prompt: None,
                tool_pipeline: vec!["generate_enterprise_noc_report".into()],
            },
            QuickCheck {
                id: "phase7_smoke".into(),
                label: "Fase 7 Smoke".into(),
                description: "Validación E2E de rendimiento y latencia.".into(),
                icon: "F7".into(),
                color: "red".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["run_phase7_smoke".into()],
                prompt: None,
                tool_pipeline: vec!["run_phase7_smoke".into()],
            },
            QuickCheck {
                id: "sla_status".into(),
                label: "SLA Status".into(),
                description: "Estado de cumplimiento de SLA por tasa de éxito histórica.".into(),
                icon: "SLA".into(),
                color: "blue".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["calculate_sla_status".into()],
                prompt: None,
                tool_pipeline: vec!["calculate_sla_status".into()],
            },
            QuickCheck {
                id: "reliability_anomalies".into(),
                label: "Reliability Anomalies".into(),
                description: "Detección proactiva de anomalías de latencia y éxito.".into(),
                icon: "ANOM".into(),
                color: "orange".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["detect_performance_anomalies".into()],
                prompt: None,
                tool_pipeline: vec!["detect_performance_anomalies".into()],
            },
            QuickCheck {
                id: "reliability_report".into(),
                label: "Reliability Report".into(),
                description: "Reporte consolidado de SLA, anomalías y acciones sugeridas.".into(),
                icon: "REL".into(),
                color: "purple".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["generate_reliability_report".into()],
                prompt: None,
                tool_pipeline: vec!["generate_reliability_report".into()],
            },
            QuickCheck {
                id: "predictive_incidents".into(),
                label: "Predictive Incidents".into(),
                description: "Predicción temprana de incidentes operacionales con probabilidad.".into(),
                icon: "PRED".into(),
                color: "purple".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["predict_operational_incidents".into()],
                prompt: None,
                tool_pipeline: vec!["predict_operational_incidents".into()],
            },
            QuickCheck {
                id: "root_cause_ai".into(),
                label: "Root Cause AI".into(),
                description: "Explicación de causa raíz con confianza para degradaciones detectadas.".into(),
                icon: "RCA".into(),
                color: "orange".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["explain_root_cause".into()],
                prompt: None,
                tool_pipeline: vec!["explain_root_cause".into()],
            },
            QuickCheck {
                id: "autonomous_playbook".into(),
                label: "Autonomous Playbook".into(),
                description: "Genera playbook autónomo de prevención y respuesta operacional.".into(),
                icon: "PLAY".into(),
                color: "blue".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["generate_autonomous_playbook".into()],
                prompt: None,
                tool_pipeline: vec!["generate_autonomous_playbook".into()],
            },
            QuickCheck {
                id: "phase8_smoke".into(),
                label: "Fase 8 Smoke".into(),
                description: "Validación E2E de fiabilidad y cumplimiento SLA.".into(),
                icon: "F8".into(),
                color: "red".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["run_phase8_smoke".into()],
                prompt: None,
                tool_pipeline: vec!["run_phase8_smoke".into()],
            },
            QuickCheck {
                id: "self_healing_readiness".into(),
                label: "Self-Healing Readiness".into(),
                description: "Evalúa nivel de preparación de autocuración por riesgo/SLA.".into(),
                icon: "SHR".into(),
                color: "blue".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["assess_self_healing_readiness".into()],
                prompt: None,
                tool_pipeline: vec!["assess_self_healing_readiness".into()],
            },
            QuickCheck {
                id: "self_healing_plan".into(),
                label: "Self-Healing Plan".into(),
                description: "Genera plan preventivo de mitigación y autocuración.".into(),
                icon: "PLAN".into(),
                color: "orange".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["generate_self_healing_plan".into()],
                prompt: None,
                tool_pipeline: vec!["generate_self_healing_plan".into()],
            },
            QuickCheck {
                id: "self_healing_runs".into(),
                label: "Self-Healing Runs".into(),
                description: "Historial de ciclos ejecutados o simulados.".into(),
                icon: "RUN".into(),
                color: "purple".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["list_self_healing_runs".into()],
                prompt: None,
                tool_pipeline: vec!["list_self_healing_runs".into()],
            },
            QuickCheck {
                id: "phase9_smoke".into(),
                label: "Fase 9 Smoke".into(),
                description: "Validación E2E de autocuración y prevención.".into(),
                icon: "F9".into(),
                color: "red".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["run_phase9_smoke".into()],
                prompt: None,
                tool_pipeline: vec!["run_phase9_smoke".into()],
            },
            QuickCheck {
                id: "go_live_readiness".into(),
                label: "Go-Live Readiness".into(),
                description: "Score consolidado de readiness para salida a producción.".into(),
                icon: "GLR".into(),
                color: "blue".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["assess_go_live_readiness".into()],
                prompt: None,
                tool_pipeline: vec!["assess_go_live_readiness".into()],
            },
            QuickCheck {
                id: "go_live_controls".into(),
                label: "Go-Live Controls".into(),
                description: "Validación de controles de seguridad y gobernanza.".into(),
                icon: "CTRL".into(),
                color: "orange".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["verify_go_live_controls".into()],
                prompt: None,
                tool_pipeline: vec!["verify_go_live_controls".into()],
            },
            QuickCheck {
                id: "go_live_bundle".into(),
                label: "Go-Live Bundle".into(),
                description: "Genera paquete de evidencia y scorecard ejecutiva.".into(),
                icon: "BUNDLE".into(),
                color: "purple".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["generate_go_live_bundle".into()],
                prompt: None,
                tool_pipeline: vec!["generate_go_live_bundle".into()],
            },
            QuickCheck {
                id: "phase10_smoke".into(),
                label: "Fase 10 Smoke".into(),
                description: "Validación E2E de cierre go-live AAA.".into(),
                icon: "F10".into(),
                color: "red".into(),
                kind: QuickCheckType::DirectTool,
                required_permissions: vec!["run_phase10_smoke".into()],
                prompt: None,
                tool_pipeline: vec!["run_phase10_smoke".into()],
            },
        ]
    }

    /// Modelos por defecto: Gemma 3 local (Docker API OpenAI) + 5 cloud gratuitos
    fn default_models() -> Vec<ModelConfig> {
        vec![
            // â”€â”€ Modelo local (Docker API OpenAI) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
            ModelConfig {
                id: "gemma3-local".into(),
                name: "Gemma 3 Local (Docker)".into(),
                provider: "docker-model-runner".into(),
                base_url: "http://localhost:11435/v1".into(),
                api_key: None,
                model_name: "gemma:2b".into(),
                supports_function_calling: true,
                is_local: true,
                max_tokens: 500,
            },
            // â”€â”€ Modelos cloud gratuitos â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
            ModelConfig {
                id: "glm-5".into(),
                name: "GLM-3 Turbo (Zhipu)".into(),
                provider: "zhipu".into(),
                base_url: "https://open.bigmodel.cn/api/paas/v4".into(),
                api_key: None,
                model_name: "glm-3-turbo".into(),
                supports_function_calling: true,
                is_local: false,
                max_tokens: 4096,
            },
            ModelConfig {
                id: "gemini-flash".into(),
                name: "Gemini 2.0 Flash".into(),
                provider: "google".into(),
                base_url: "https://generativelanguage.googleapis.com/v1beta/openai".into(),
                api_key: None,
                model_name: "gemini-2.0-flash".into(),
                supports_function_calling: true,
                is_local: false,
                max_tokens: 8192,
            },
            ModelConfig {
                id: "llama-groq".into(),
                name: "Llama 3.3 70B (Groq)".into(),
                provider: "groq".into(),
                base_url: "https://api.groq.com/openai/v1".into(),
                api_key: None,
                model_name: "llama-3.3-70b-versatile".into(),
                supports_function_calling: true,
                is_local: false,
                max_tokens: 4096,
            },
            ModelConfig {
                id: "qwen-openrouter".into(),
                name: "Qwen 2.5 72B".into(),
                provider: "openrouter".into(),
                base_url: "https://openrouter.ai/api/v1".into(),
                api_key: None,
                model_name: "qwen/qwen-2.5-72b-instruct".into(),
                supports_function_calling: true,
                is_local: false,
                max_tokens: 4096,
            },
            ModelConfig {
                id: "deepseek-v3".into(),
                name: "DeepSeek V3".into(),
                provider: "deepseek".into(),
                base_url: "https://api.deepseek.com/v1".into(),
                api_key: None,
                model_name: "deepseek-chat".into(),
                supports_function_calling: true,
                is_local: false,
                max_tokens: 4096,
            },
        ]
    }
}
