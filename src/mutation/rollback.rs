use crate::source::Endpoint;
use std::collections::BTreeMap;

/// A compensating rollback action triggered if a subsequent mutation step fails
#[derive(Clone)]
pub struct CompensatingRollback {
    pub step_name: String,
    pub rollback_endpoint: Endpoint,
    pub path_mappings: BTreeMap<String, String>,
}

impl CompensatingRollback {
    pub fn new(step_name: impl Into<String>, rollback_endpoint: Endpoint) -> Self {
        Self {
            step_name: step_name.into(),
            rollback_endpoint,
            path_mappings: BTreeMap::new(),
        }
    }

    pub fn map_path(mut self, target_param: impl Into<String>, source_path: impl Into<String>) -> Self {
        self.path_mappings.insert(target_param.into(), source_path.into());
        self
    }
}
