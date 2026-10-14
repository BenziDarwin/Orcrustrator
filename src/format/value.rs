use crate::error::{OrcrustratorError, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// Dynamic unified in-memory representation for payloads across JSON, XML, etc.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StructuredValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<StructuredValue>),
    Object(BTreeMap<String, StructuredValue>),
}

impl StructuredValue {
    /// Create an empty object
    pub fn empty_object() -> Self {
        StructuredValue::Object(BTreeMap::new())
    }

    /// Create an empty array
    pub fn empty_array() -> Self {
        StructuredValue::Array(Vec::new())
    }

    /// Check if null
    pub fn is_null(&self) -> bool {
        matches!(self, StructuredValue::Null)
    }

    /// As object reference
    pub fn as_object(&self) -> Option<&BTreeMap<String, StructuredValue>> {
        match self {
            StructuredValue::Object(map) => Some(map),
            _ => None,
        }
    }

    /// As mutable object reference
    pub fn as_object_mut(&mut self) -> Option<&mut BTreeMap<String, StructuredValue>> {
        match self {
            StructuredValue::Object(map) => Some(map),
            _ => None,
        }
    }

    /// As array reference
    pub fn as_array(&self) -> Option<&Vec<StructuredValue>> {
        match self {
            StructuredValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// As mutable array reference
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<StructuredValue>> {
        match self {
            StructuredValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// As string slice
    pub fn as_str(&self) -> Option<&str> {
        match self {
            StructuredValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Retrieve a nested value by dot/JSON path, e.g. "user.profile.id" or "$.user.id"
    pub fn get_path(&self, path: &str) -> Option<&StructuredValue> {
        let clean_path = path
            .strip_prefix("$.")
            .or_else(|| path.strip_prefix('$'))
            .unwrap_or(path);
        if clean_path.is_empty() {
            return Some(self);
        }

        let parts: Vec<&str> = clean_path.split('.').collect();
        let mut current = self;

        for part in parts {
            match current {
                StructuredValue::Object(map) => {
                    current = map.get(part)?;
                }
                StructuredValue::Array(arr) => {
                    if let Ok(idx) = part.parse::<usize>() {
                        current = arr.get(idx)?;
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }

        Some(current)
    }

    /// Set a nested value by dot path, e.g. "user.company"
    pub fn set_path(&mut self, path: &str, value: StructuredValue) {
        let clean_path = path
            .strip_prefix("$.")
            .or_else(|| path.strip_prefix('$'))
            .unwrap_or(path);
        let parts: Vec<&str> = clean_path.split('.').collect();
        if parts.is_empty() {
            *self = value;
            return;
        }

        if !matches!(self, StructuredValue::Object(_)) {
            *self = StructuredValue::Object(BTreeMap::new());
        }

        let mut current = self;
        for &part in parts.iter().take(parts.len().saturating_sub(1)) {
            if let StructuredValue::Object(map) = current {
                current = map.entry(part.to_string()).or_insert_with(|| StructuredValue::Object(BTreeMap::new()));
            }
        }

        if let Some(last_key) = parts.last()
            && let StructuredValue::Object(map) = current
        {
            map.insert(last_key.to_string(), value);
        }
    }

    /// Convert into serde_json::Value
    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            StructuredValue::Null => serde_json::Value::Null,
            StructuredValue::Bool(b) => serde_json::Value::Bool(*b),
            StructuredValue::Number(n) => {
                if n.fract() == 0.0 && *n >= 0.0 && *n <= u64::MAX as f64 {
                    serde_json::Value::Number(serde_json::Number::from(*n as u64))
                } else if n.fract() == 0.0 && *n < 0.0 && *n >= i64::MIN as f64 {
                    serde_json::Value::Number(serde_json::Number::from(*n as i64))
                } else if let Some(num) = serde_json::Number::from_f64(*n) {
                    serde_json::Value::Number(num)
                } else {
                    serde_json::Value::Null
                }
            }
            StructuredValue::String(s) => serde_json::Value::String(s.clone()),
            StructuredValue::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(|v| v.to_json_value()).collect())
            }
            StructuredValue::Object(map) => {
                let mut json_map = serde_json::Map::new();
                for (k, v) in map {
                    json_map.insert(k.clone(), v.to_json_value());
                }
                serde_json::Value::Object(json_map)
            }
        }
    }

    /// Create from serde_json::Value
    pub fn from_json_value(val: serde_json::Value) -> Self {
        match val {
            serde_json::Value::Null => StructuredValue::Null,
            serde_json::Value::Bool(b) => StructuredValue::Bool(b),
            serde_json::Value::Number(n) => {
                StructuredValue::Number(n.as_f64().unwrap_or(0.0))
            }
            serde_json::Value::String(s) => StructuredValue::String(s),
            serde_json::Value::Array(arr) => {
                StructuredValue::Array(arr.into_iter().map(StructuredValue::from_json_value).collect())
            }
            serde_json::Value::Object(map) => {
                let mut btree = BTreeMap::new();
                for (k, v) in map {
                    btree.insert(k, StructuredValue::from_json_value(v));
                }
                StructuredValue::Object(btree)
            }
        }
    }

    /// Render to JSON string
    pub fn to_json_string(&self) -> Result<String> {
        serde_json::to_string(&self.to_json_value())
            .map_err(OrcrustratorError::from)
    }

    /// Render to pretty JSON string
    pub fn to_json_string_pretty(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.to_json_value())
            .map_err(OrcrustratorError::from)
    }

    /// Strongly-typed deserialization for crate users
    pub fn into_typed<T: DeserializeOwned>(&self) -> Result<T> {
        let json_val = self.to_json_value();
        serde_json::from_value(json_val).map_err(OrcrustratorError::from)
    }
}

impl fmt::Display for StructuredValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.to_json_string() {
            Ok(s) => write!(f, "{}", s),
            Err(_) => write!(f, "<unserializable StructuredValue>"),
        }
    }
}

impl From<serde_json::Value> for StructuredValue {
    fn from(val: serde_json::Value) -> Self {
        StructuredValue::from_json_value(val)
    }
}

impl From<&str> for StructuredValue {
    fn from(s: &str) -> Self {
        StructuredValue::String(s.to_string())
    }
}

impl From<String> for StructuredValue {
    fn from(s: String) -> Self {
        StructuredValue::String(s)
    }
}

impl From<bool> for StructuredValue {
    fn from(b: bool) -> Self {
        StructuredValue::Bool(b)
    }
}

impl From<i64> for StructuredValue {
    fn from(n: i64) -> Self {
        StructuredValue::Number(n as f64)
    }
}

impl From<f64> for StructuredValue {
    fn from(n: f64) -> Self {
        StructuredValue::Number(n)
    }
}
