use std::collections::HashMap;

pub fn extract_map(t: &str) -> HashMap<String, String> {
    t.split(':')
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .map(|t| {
            let parts: Vec<String> = t.split('=').map(|f| f.replace('\"', "")).collect();
            (parts[0].to_string(), parts[1].to_string())
        })
        .collect()
}
