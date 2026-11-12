use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

use crate::storage;

pub fn read_json_object_file(path: &Path) -> Result<HashMap<String, Value>, String> {
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let s = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {e}"))?;
    if s.trim().is_empty() {
        return Ok(HashMap::new());
    }
    let v: Value = serde_json::from_str(&s).map_err(|e| format!("Invalid JSON: {e}"))?;
    match v {
        Value::Object(map) => Ok(map.into_iter().collect()),
        _ => Err("Invalid JSON: expected object".to_string()),
    }
}

pub fn write_json_object_file(path: &Path, obj: &HashMap<String, Value>) -> Result<(), String> {
    let v = Value::Object(obj.clone().into_iter().collect());
    let s =
        serde_json::to_string_pretty(&v).map_err(|e| format!("Failed to serialize JSON: {e}"))?;
    storage::atomic_write(path, s.as_bytes())
}
