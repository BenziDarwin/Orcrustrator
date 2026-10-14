use crate::format::StructuredValue;

/// Extracts dynamic parameter values from a StructuredValue using dot paths
pub struct KeyExtractor;

impl KeyExtractor {
    /// Extract a single value as a String (e.g. "company_id" from "user.company_id" or "company_id")
    pub fn extract_string(value: &StructuredValue, path: &str) -> Option<String> {
        let val = value.get_path(path)?;
        match val {
            StructuredValue::String(s) => Some(s.clone()),
            StructuredValue::Number(n) => Some(n.to_string()),
            StructuredValue::Bool(b) => Some(b.to_string()),
            _ => None,
        }
    }

    /// Extract multiple values (e.g. if path points to an array of IDs or numbers/strings)
    pub fn extract_list(value: &StructuredValue, path: &str) -> Vec<String> {
        if let Some(val) = value.get_path(path) {
            match val {
                StructuredValue::Array(arr) => arr
                    .iter()
                    .filter_map(|item| match item {
                        StructuredValue::String(s) => Some(s.clone()),
                        StructuredValue::Number(n) => Some(n.to_string()),
                        _ => None,
                    })
                    .collect(),
                StructuredValue::String(s) => vec![s.clone()],
                StructuredValue::Number(n) => vec![n.to_string()],
                _ => Vec::new(),
            }
        } else {
            Vec::new()
        }
    }
}
