use super::ToolResult;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

fn to_ascii_safe(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut last_was_underscore = false;

    for ch in input.chars() {
        let mapped =
            if ch.is_ascii_alphanumeric() || matches!(ch, ' ' | '.' | '_' | '-' | '(' | ')') {
                ch
            } else {
                '_'
            };

        if mapped == '_' {
            if !last_was_underscore {
                out.push(mapped);
            }
            last_was_underscore = true;
        } else {
            out.push(mapped);
            last_was_underscore = false;
        }
    }

    out.trim().to_string()
}

fn resolve_path(input: &str) -> PathBuf {
    let trimmed = input.trim();

    if trimmed.is_empty() || trimmed == "." {
        return std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    }

    let lowered = trimmed.to_lowercase();
    if lowered == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }

    if lowered == "desktop"
        || lowered == "escritorio"
        || lowered == "~/desktop"
        || lowered == "~/escritorio"
    {
        if let Some(desktop) = dirs::desktop_dir() {
            return desktop;
        }
        if let Some(home) = dirs::home_dir() {
            return home.join("Desktop");
        }
    }

    let candidate = PathBuf::from(trimmed);
    if candidate.is_absolute() {
        return candidate;
    }

    if let Some(home) = dirs::home_dir() {
        let maybe_home_relative = home.join(trimmed);
        if maybe_home_relative.exists() {
            return maybe_home_relative;
        }
    }

    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(trimmed)
}

/// Lee el contenido de un archivo de texto
pub fn read_file(path: &str) -> ToolResult {
    log::info!("Reading file: {}", path);

    let resolved_path = resolve_path(path);
    let file_path = resolved_path.as_path();

    if !file_path.exists() {
        return ToolResult {
            tool_name: "read_file".to_string(),
            success: false,
            output: String::new(),
            error: Some(format!("File not found: {}", file_path.display())),
        };
    }

    // Verificar tamaÃ±o (mÃ¡ximo 1MB para lectura)
    if let Ok(metadata) = fs::metadata(file_path) {
        if metadata.len() > 1_048_576 {
            return ToolResult {
                tool_name: "read_file".to_string(),
                success: false,
                output: String::new(),
                error: Some(format!(
                    "File too large: {} bytes (max 1MB)",
                    metadata.len()
                )),
            };
        }
    }

    match fs::read_to_string(file_path) {
        Ok(content) => ToolResult {
            tool_name: "read_file".to_string(),
            success: true,
            output: content,
            error: None,
        },
        Err(e) => ToolResult {
            tool_name: "read_file".to_string(),
            success: false,
            output: String::new(),
            error: Some(match e.kind() {
                ErrorKind::PermissionDenied => {
                    format!(
                        "Permission denied while reading file: {}",
                        file_path.display()
                    )
                }
                _ => format!("Error reading file '{}': {}", file_path.display(), e),
            }),
        },
    }
}

/// Crea o sobrescribe un archivo con el contenido proporcionado
pub fn write_file(path: &str, content: &str) -> ToolResult {
    log::info!("Writing file: {}", path);

    let resolved_path = resolve_path(path);
    let file_path = resolved_path.as_path();

    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            if let Err(e) = fs::create_dir_all(parent) {
                return ToolResult {
                    tool_name: "write_file".to_string(),
                    success: false,
                    output: String::new(),
                    error: Some(format!(
                        "Error creating directories for '{}': {}",
                        file_path.display(),
                        e
                    )),
                };
            }
        }
    }

    match fs::write(file_path, content) {
        Ok(_) => ToolResult {
            tool_name: "write_file".to_string(),
            success: true,
            output: format!(
                "File written successfully: {} ({} bytes)",
                file_path.display(),
                content.len()
            ),
            error: None,
        },
        Err(e) => ToolResult {
            tool_name: "write_file".to_string(),
            success: false,
            output: String::new(),
            error: Some(match e.kind() {
                ErrorKind::PermissionDenied => format!(
                    "Permission denied while writing file: {}",
                    file_path.display()
                ),
                _ => format!("Error writing file '{}': {}", file_path.display(), e),
            }),
        },
    }
}

/// Lista los archivos y carpetas de un directorio
pub fn list_directory(path: &str) -> ToolResult {
    log::info!("Listing directory: {}", path);

    let resolved_path = resolve_path(path);
    let dir_path = resolved_path.as_path();

    if !dir_path.exists() {
        return ToolResult {
            tool_name: "list_directory".to_string(),
            success: false,
            output: String::new(),
            error: Some(format!("Directory not found: {}", dir_path.display())),
        };
    }

    if !dir_path.is_dir() {
        return ToolResult {
            tool_name: "list_directory".to_string(),
            success: false,
            output: String::new(),
            error: Some(format!("Not a directory: {}", dir_path.display())),
        };
    }

    match fs::read_dir(dir_path) {
        Ok(entries) => {
            let mut items: Vec<String> = Vec::new();
            let mut file_count = 0u32;
            let mut dir_count = 0u32;

            for entry in entries.flatten() {
                let raw_name = entry.file_name().to_string_lossy().to_string();
                let name = to_ascii_safe(&raw_name);
                let metadata = entry.metadata();

                let (type_indicator, size_str) = if let Ok(meta) = &metadata {
                    if meta.is_dir() {
                        dir_count += 1;
                        ("[DIR]", String::new())
                    } else {
                        file_count += 1;
                        let size = meta.len();
                        let size_formatted = if size < 1024 {
                            format!("{} B", size)
                        } else if size < 1_048_576 {
                            format!("{:.1} KB", size as f64 / 1024.0)
                        } else if size < 1_073_741_824 {
                            format!("{:.1} MB", size as f64 / 1_048_576.0)
                        } else {
                            format!("{:.1} GB", size as f64 / 1_073_741_824.0)
                        };
                        ("[FILE]", format!("  ({})", size_formatted))
                    }
                } else {
                    ("[?]", String::new())
                };

                items.push(format!("{} {}{}", type_indicator, name, size_str));
            }

            items.sort();
            let summary = format!("\n--- {} files, {} directories ---", file_count, dir_count);
            items.push(summary);

            ToolResult {
                tool_name: "list_directory".to_string(),
                success: true,
                output: items.join("\n"),
                error: None,
            }
        }
        Err(e) => ToolResult {
            tool_name: "list_directory".to_string(),
            success: false,
            output: String::new(),
            error: Some(match e.kind() {
                ErrorKind::PermissionDenied => {
                    format!(
                        "Permission denied while listing directory: {}",
                        dir_path.display()
                    )
                }
                _ => format!("Error reading directory '{}': {}", dir_path.display(), e),
            }),
        },
    }
}
