use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationRequest {
    pub session_id: String,
    pub user_id: String,
    pub current_role: String,
    pub required_role: String,
    pub action_name: String,
    pub risk_level: String,
    pub password_input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationResponse {
    pub success: bool,
    pub authenticated_role: String,
    pub message: String,
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInteractionRecord {
    pub id: String,
    pub session_id: String,
    pub user_id: String,
    pub user_role: String,
    pub query_text: String,
    pub intent_detected: String,
    pub response_mode: String,
    pub action_requested: Option<String>,
    pub command_risk_level: Option<String>,
    pub elevation_required: bool,
    pub elevation_status: String,
    pub authenticated_by: Option<String>,
    pub execution_result: Option<String>,
}

pub fn verify_technical_password(password: &str, required_role: &str) -> ElevationResponse {
    let trimmed = password.trim();

    // Contraseñas predeterminadas de evaluación para usuarios de soporte y superadmin
    if trimmed == "admin123" || trimmed == "superadmin123" || trimmed == "kernelia2026" {
        let granted_role = if trimmed == "superadmin123" {
            "superadmin"
        } else {
            "tech_analyst"
        };

        ElevationResponse {
            success: true,
            authenticated_role: granted_role.to_string(),
            message: format!("Autenticación exitosa como {granted_role}. Elevación de privilegios concedida."),
            error_code: None,
        }
    } else {
        ElevationResponse {
            success: false,
            authenticated_role: required_role.to_string(),
            message: "Contraseña de usuario técnico o administrador incorrecta.".to_string(),
            error_code: Some("INVALID_CREDENTIALS".to_string()),
        }
    }
}

pub fn evaluate_risk_level(action_name: &str) -> &'static str {
    let lower = action_name.to_lowercase();
    if lower.contains("format") || lower.contains("remove-item") || lower.contains("delete") || lower.contains("del ") {
        "R4"
    } else if lower.contains("registry") || lower.contains("uninstall") || lower.contains("reboot") {
        "R3"
    } else if lower.contains("restart-service") || lower.contains("reset") {
        "R2"
    } else if lower.contains("sfc") || lower.contains("flushdns") {
        "R1"
    } else {
        "R0"
    }
}
