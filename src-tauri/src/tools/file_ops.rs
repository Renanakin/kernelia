use crate::tools::ToolResult;
use std::fs;

pub fn read_file(path: &str) -> ToolResult {
    match fs::read_to_string(path) {
        Ok(content) => ToolResult {
            tool_name: "read_file_ops".into(),
            success: true,
            output: content,
            error: None,
        },
        Err(e) => ToolResult {
            tool_name: "read_file_ops".into(),
            success: false,
            output: String::new(),
            error: Some(e.to_string()),
        },
    }
}

pub fn write_file(path: &str, content: &str) -> ToolResult {
    match fs::write(path, content) {
        Ok(_) => ToolResult {
            tool_name: "write_file_ops".into(),
            success: true,
            output: format!("Archivo escrito: {}", path),
            error: None,
        },
        Err(e) => ToolResult {
            tool_name: "write_file_ops".into(),
            success: false,
            output: String::new(),
            error: Some(e.to_string()),
        },
    }
}
