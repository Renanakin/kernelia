use serde_json::Value;

fn parse_tool_calls_from_text(text: &str) {
    let mut calls = Vec::new();
    if let Some(start_idx) = text.find("```json") {
        let content = &text[start_idx + 7..];
        if let Some(end_idx) = content.find("```") {
            let json_str = content[..end_idx].trim();
            println!("Extracted JSON: {}", json_str);
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                if let Some(name) = val.get("name").and_then(|v| v.as_str()) {
                    let arguments = val.get("arguments")
                        .map(|v| if v.is_string() { v.as_str().unwrap().to_string() } else { v.to_string() })
                        .unwrap_or_else(|| "{}".to_string());
                    println!("Name: {}, Args: {}", name, arguments);
                    calls.push(name.to_string());
                } else {
                    println!("No name found");
                }
            } else {
                println!("Failed to parse JSON");
            }
        }
    }
    
    if calls.is_empty() {
        if let Some(start_idx) = text.find('{') {
            if let Some(end_idx) = text.rfind('}') {
                let json_str = &text[start_idx..=end_idx];
                println!("Fallback JSON: {}", json_str);
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let Some(name) = val.get("name").and_then(|v| v.as_str()) {
                        let arguments = val.get("arguments")
                            .map(|v| if v.is_string() { v.as_str().unwrap().to_string() } else { v.to_string() })
                            .unwrap_or_else(|| "{}".to_string());
                        println!("Name: {}, Args: {}", name, arguments);
                        calls.push(name.to_string());
                    } else {
                        println!("Fallback No name found");
                    }
                } else {
                    println!("Fallback Failed to parse JSON");
                }
            }
        }
    }
}

fn main() {
    let text = "```json\n{\n  \"name\": \"run_network_diagnostic\",\n  \"arguments\": \"{}\"\n}\n```";
    parse_tool_calls_from_text(text);
    
    let text2 = "```json\n{\n  \"name\": \"run_network_diagnostic\",\n  \"arguments\": {}\n}\n```";
    parse_tool_calls_from_text(text2);
    
    let text3 = "Here is the tool: { \"name\": \"run_network_diagnostic\" }";
    parse_tool_calls_from_text(text3);
}
