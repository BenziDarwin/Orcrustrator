use crate::source::Endpoint;
use std::collections::BTreeMap;

/// A single step in a multi-step mutation pipeline
#[derive(Clone)]
pub struct MutationStep {
    pub name: String,
    pub endpoint: Endpoint,
    /// Maps target body field -> source path (e.g. "name" -> "$.company_name" or "$steps.create_org.id")
    pub body_mappings: BTreeMap<String, String>,
    /// Maps target path parameter -> source path (e.g. "org_id" -> "$steps.create_org.id")
    pub path_mappings: BTreeMap<String, String>,
}

impl MutationStep {
    pub fn new(name: impl Into<String>, endpoint: Endpoint) -> Self {
        Self {
            name: name.into(),
            endpoint,
            body_mappings: BTreeMap::new(),
            path_mappings: BTreeMap::new(),
        }
    }

    pub fn map_body(mut self, target_field: impl Into<String>, source_path: impl Into<String>) -> Self {
        self.body_mappings.insert(target_field.into(), source_path.into());
        self
    }

    pub fn map_path(mut self, target_param: impl Into<String>, source_path: impl Into<String>) -> Self {
        self.path_mappings.insert(target_param.into(), source_path.into());
        self
    }
}
