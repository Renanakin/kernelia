use super::{security, ToolResult};
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Clone)]
pub enum CleanupArea {
    UserTemp,
    SystemTemp,
    Prefetch,
    RecycleBin,
}

impl CleanupArea {
    fn get_path(&self) -> Option<PathBuf> {
        match self {
            CleanupArea::UserTemp => std::env::var_os("TEMP").map(PathBuf::from),
            CleanupArea::SystemTemp => Some(PathBuf::from("C:\\Windows\\Temp")),
            CleanupArea::Prefetch => Some(PathBuf::from("C:\\Windows\\Prefetch")),
            CleanupArea::RecycleBin => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            CleanupArea::UserTemp => "Temporales de Usuario",
            CleanupArea::SystemTemp => "Temporales del Sistema",
            CleanupArea::Prefetch => "Cache de Prefetch",
            CleanupArea::RecycleBin => "Papelera de Reciclaje",
        }
    }
}

pub fn analyze_junk() -> ToolResult {
    let areas = vec![
        CleanupArea::UserTemp,
        CleanupArea::SystemTemp,
        CleanupArea::Prefetch,
    ];
    let mut total_size = 0u64;
    let mut details = Vec::new();

    for area in areas {
        if let Some(path) = area.get_path() {
            if path.exists() {
                let size = get_dir_size(&path).unwrap_or(0);
                total_size += size;
                details.push(format!("{}: {}", area.as_str(), format_size(size)));
            }
        }
    }

    ToolResult {
        tool_name: "analyze_junk".to_string(),
        success: true,
        output: format!(
            "Analisis de limpieza completado.\n\nDetalles:\n- {}\n\nTotal recuperable estimado: {}",
            details.join("\n- "),
            format_size(total_size)
        ),
        error: None,
    }
}

pub async fn run_cleanup(target_areas: Option<Vec<String>>) -> ToolResult {
    let mut results = Vec::new();
    let mut total_freed = 0u64;
    let areas_to_clean = if let Some(tags) = target_areas {
        tags.iter()
            .filter_map(|t| match t.to_lowercase().as_str() {
                "user_temp" | "temp" => Some(CleanupArea::UserTemp),
                "system_temp" => Some(CleanupArea::SystemTemp),
                "prefetch" => Some(CleanupArea::Prefetch),
                "recycle" | "trash" => Some(CleanupArea::RecycleBin),
                _ => None,
            })
            .collect()
    } else {
        vec![
            CleanupArea::UserTemp,
            CleanupArea::SystemTemp,
            CleanupArea::Prefetch,
            CleanupArea::RecycleBin,
        ]
    };

    for area in areas_to_clean {
        match area {
            CleanupArea::RecycleBin => {
                let mut command = tokio::process::Command::new("powershell.exe");
                command.args([
                    "-NoProfile",
                    "-Command",
                    "Clear-RecycleBin -Confirm:$false -ErrorAction SilentlyContinue",
                ]);
                #[cfg(windows)]
                command.creation_flags(CREATE_NO_WINDOW);
                let output = command.output().await;
                if output.is_ok() {
                    results.push(format!("{}: Vaciada", area.as_str()));
                } else {
                    results.push(format!("{}: Error al vaciar", area.as_str()));
                }
            }
            _ => {
                if let Some(path) = area.get_path() {
                    if let Err(e) = security::validate_path(&path, true) {
                        results.push(format!("{}: Bloqueado ({})", area.as_str(), e));
                        continue;
                    }

                    match empty_dir(&path) {
                        Ok(freed) => {
                            total_freed += freed;
                            results.push(format!(
                                "{}: Liberados {}",
                                area.as_str(),
                                format_size(freed)
                            ));
                        }
                        Err(e) => {
                            results.push(format!("{}: Parcialmente limpio. {}", area.as_str(), e));
                        }
                    }
                }
            }
        }
    }

    ToolResult {
        tool_name: "run_cleanup".to_string(),
        success: true,
        output: format!(
            "Operacion de limpieza finalizada.\n\nResultado:\n{}\n\nEspacio total recuperado: {}",
            results.join("\n"),
            format_size(total_freed)
        ),
        error: None,
    }
}

fn get_dir_size(path: &Path) -> std::io::Result<u64> {
    let mut size = 0;
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                size += get_dir_size(&path).unwrap_or(0);
            } else {
                size += entry.metadata()?.len();
            }
        }
    }
    Ok(size)
}

fn empty_dir(path: &Path) -> std::io::Result<u64> {
    let mut freed = 0;
    if !path.exists() {
        return Ok(0);
    }

    for entry in fs::read_dir(path)? {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_dir() {
                let size = get_dir_size(&path).unwrap_or(0);
                if fs::remove_dir_all(&path).is_ok() {
                    freed += size;
                }
            } else if let Ok(meta) = entry.metadata() {
                let size = meta.len();
                if fs::remove_file(&path).is_ok() {
                    freed += size;
                }
            }
        }
    }
    Ok(freed)
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
