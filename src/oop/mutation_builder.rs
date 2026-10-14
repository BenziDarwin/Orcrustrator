use crate::method::HttpMethod;
use crate::mutation::{CompensatingRollback, MutationEndpoint, MutationStep};
use crate::source::Endpoint;
use std::collections::BTreeMap;

/// Fluent OOP builder for constructing MutationEndpoint (POST, PUT, PATCH, DELETE) instances
pub struct MutationEndpointBuilder {
    method: HttpMethod,
    path_pattern: String,
    steps: Vec<MutationStep>,
    rollbacks: BTreeMap<String, CompensatingRollback>,
    response_mappings: BTreeMap<String, String>,
}

impl MutationEndpointBuilder {
    pub fn new(method: HttpMethod, path_pattern: impl Into<String>) -> Self {
        Self {
            method,
            path_pattern: path_pattern.into(),
            steps: Vec::new(),
            rollbacks: BTreeMap::new(),
            response_mappings: BTreeMap::new(),
        }
    }

    pub fn post(path_pattern: impl Into<String>) -> Self {
        Self::new(HttpMethod::Post, path_pattern)
    }

    pub fn put(path_pattern: impl Into<String>) -> Self {
        Self::new(HttpMethod::Put, path_pattern)
    }

    pub fn patch(path_pattern: impl Into<String>) -> Self {
        Self::new(HttpMethod::Patch, path_pattern)
    }

    pub fn delete(path_pattern: impl Into<String>) -> Self {
        Self::new(HttpMethod::Delete, path_pattern)
    }

    /// Add a step with a custom step builder
    pub fn step(mut self, step: MutationStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Add a compensating rollback action for a step
    pub fn rollback(mut self, step_name: impl Into<String>, rollback_endpoint: Endpoint) -> Self {
        let name = step_name.into();
        self.rollbacks.insert(
            name.clone(),
            CompensatingRollback::new(name, rollback_endpoint),
        );
        self
    }

    /// Add a compensating rollback with path mapping
    pub fn rollback_with_mapping(mut self, rollback: CompensatingRollback) -> Self {
        self.rollbacks.insert(rollback.step_name.clone(), rollback);
        self
    }

    /// Map a response field
    pub fn map_response(mut self, target_field: impl Into<String>, source_path: impl Into<String>) -> Self {
        self.response_mappings.insert(target_field.into(), source_path.into());
        self
    }

    /// Build the MutationEndpoint
    pub fn build(self) -> MutationEndpoint {
        MutationEndpoint {
            path_pattern: self.path_pattern,
            method: self.method,
            steps: self.steps,
            rollbacks: self.rollbacks,
            response_mappings: self.response_mappings,
        }
    }
}
