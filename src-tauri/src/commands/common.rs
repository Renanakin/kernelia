pub const MAX_MESSAGE_LEN: usize = 12_000;
pub const MAX_USERNAME_LEN: usize = 64;
pub const MAX_PASSWORD_LEN: usize = 256;
pub const MAX_MODEL_ID_LEN: usize = 120;
pub const MAX_TOOL_NAME_LEN: usize = 120;
pub const MAX_SERVICE_NAME_LEN: usize = 120;
pub const MAX_QUICK_CHECK_ID_LEN: usize = 120;

pub fn validate_non_empty(field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("API_VALIDATION: '{}' no puede estar vacio", field));
    }
    Ok(())
}

pub fn validate_max_len(field: &str, value: &str, max: usize) -> Result<(), String> {
    if value.chars().count() > max {
        return Err(format!(
            "API_VALIDATION: '{}' excede longitud maxima ({})",
            field, max
        ));
    }
    Ok(())
}

pub fn validate_message(message: &str) -> Result<(), String> {
    validate_non_empty("message", message)?;
    validate_max_len("message", message, MAX_MESSAGE_LEN)
}

pub fn validate_username(username: &str) -> Result<(), String> {
    validate_non_empty("username", username)?;
    validate_max_len("username", username, MAX_USERNAME_LEN)
}

pub fn validate_password(password: &str, field_name: &str) -> Result<(), String> {
    validate_non_empty(field_name, password)?;
    validate_max_len(field_name, password, MAX_PASSWORD_LEN)
}

pub fn validate_model_id(model_id: &str) -> Result<(), String> {
    validate_non_empty("model_id", model_id)?;
    validate_max_len("model_id", model_id, MAX_MODEL_ID_LEN)
}

pub fn validate_tool_name(tool_name: &str) -> Result<(), String> {
    validate_non_empty("name", tool_name)?;
    validate_max_len("name", tool_name, MAX_TOOL_NAME_LEN)
}

pub fn validate_service_name(service_name: &str) -> Result<(), String> {
    validate_non_empty("service_name", service_name)?;
    validate_max_len("service_name", service_name, MAX_SERVICE_NAME_LEN)
}

pub fn validate_quick_check_id(quick_check_id: &str) -> Result<(), String> {
    validate_non_empty("quick_check_id", quick_check_id)?;
    validate_max_len("quick_check_id", quick_check_id, MAX_QUICK_CHECK_ID_LEN)
}

pub fn normalize_sort_by(sort_by: Option<String>) -> Result<String, String> {
    let normalized = sort_by
        .unwrap_or_else(|| "memory".to_string())
        .trim()
        .to_lowercase();

    match normalized.as_str() {
        "cpu" | "memory" | "name" => Ok(normalized),
        _ => Err("API_VALIDATION: 'sort_by' debe ser uno de: cpu, memory, name".to_string()),
    }
}

pub fn normalize_limit(limit: Option<usize>, default_value: usize, max_value: usize) -> usize {
    limit.unwrap_or(default_value).clamp(1, max_value)
}
