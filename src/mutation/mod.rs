pub mod pipeline;
pub mod rollback;
pub mod step;

pub use pipeline::MutationPipeline;
pub use rollback::CompensatingRollback;
pub use step::MutationStep;

use crate::method::HttpMethod;
use std::collections::BTreeMap;

/// A composite mutation endpoint definition for POST, PUT, PATCH, DELETE operations
#[derive(Clone)]
pub struct MutationEndpoint {
    pub path_pattern: String,
    pub method: HttpMethod,
    pub steps: Vec<MutationStep>,
    pub rollbacks: BTreeMap<String, CompensatingRollback>,
    pub response_mappings: BTreeMap<String, String>,
}

impl MutationEndpoint {
    pub fn new(method: HttpMethod, path_pattern: impl Into<String>) -> Self {
        Self {
            path_pattern: path_pattern.into(),
            method,
            steps: Vec::new(),
            rollbacks: BTreeMap::new(),
            response_mappings: BTreeMap::new(),
        }
    }

    pub fn add_step(&mut self, step: MutationStep) {
        self.steps.push(step);
    }

    pub fn add_rollback(&mut self, rollback: CompensatingRollback) {
        self.rollbacks.insert(rollback.step_name.clone(), rollback);
    }

    pub fn map_response(&mut self, target_field: impl Into<String>, source_path: impl Into<String>) {
        self.response_mappings.insert(target_field.into(), source_path.into());
    }
}
