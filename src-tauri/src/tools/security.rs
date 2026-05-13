use std::path::{Path, PathBuf};

const FORBIDDEN_PATTERNS: &[&str] = &[
    "rm -rf",
    "format ",
    "mkfs",
    "shutdown",
    "reboot",
    "del /s",
    "rd /s",
    "net user",
    "net share",
    "Invoke-WebRequest",
    "wget",
    "curl",
    "chmod -R 777",
    "chown",
    "passwd",
    "rmdir /s",
    "attrib -r -s -h",
    "reg delete",
    "taskkill /f /im explorer.exe",
    "stop-process -name explorer",
    "Invoke-Expression",
    "iex",
];

const PROTECTED_PATHS: &[&str] = &[
    "C:\\Windows",
    "C:\\Program Files",
    "C:\\Program Files (x86)",
    "C:\\Users\\All Users",
    "/etc",
    "/bin",
    "/sbin",
    "/usr",
    "/boot",
];

pub fn validate_command(command: &str) -> Result<(), String> {
    let cmd_lower = command.to_lowercase();
    for pattern in FORBIDDEN_PATTERNS {
        if cmd_lower.contains(&pattern.to_lowercase()) {
            return Err(format!(
                "COMANDO BLOQUEADO POR SEGURIDAD: El patron '{}' esta prohibido.",
                pattern
            ));
        }
    }
    Ok(())
}

pub fn validate_path(path: &Path, is_write: bool) -> Result<(), String> {
    if is_write {
        let path_str = path.to_string_lossy();
        for protected in PROTECTED_PATHS {
            if path_str.starts_with(protected) {
                return Err(format!(
                    "RUTA PROTEGIDA: escritura bloqueada en '{}'.",
                    protected
                ));
            }
        }
    }

    Ok(())
}
