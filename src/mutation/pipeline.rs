use crate::error::{OrcrustratorError, Result};
use crate::format::StructuredValue;
use crate::mutation::MutationEndpoint;
use crate::source::HttpSourceClient;
use std::collections::{BTreeMap, HashMap};

/// Executes multi-step mutation pipelines with parameter forwarding and rollbacks
pub struct MutationPipeline;

impl MutationPipeline {
    /// Resolve a value from either client_body or previous step results
    fn resolve_value(
        source_path: &str,
        client_body: Option<&StructuredValue>,
        client_params: &HashMap<String, String>,
        step_results: &BTreeMap<String, StructuredValue>,
    ) -> Option<StructuredValue> {
        if source_path.starts_with("$steps.") {
            let path_without_prefix = source_path.strip_prefix("$steps.")?;
            let parts: Vec<&str> = path_without_prefix.splitn(2, '.').collect();
            let step_name = parts[0];
            let step_val = step_results.get(step_name)?;

            if parts.len() > 1 {
                step_val.get_path(parts[1]).cloned()
            } else {
                Some(step_val.clone())
            }
        } else if source_path.starts_with("$params.") {
            let param_name = source_path.strip_prefix("$params.")?;
            client_params.get(param_name).map(|v| StructuredValue::String(v.clone()))
        } else if let Some(body) = client_body {
            body.get_path(source_path).cloned()
        } else {
            None
        }
    }

    /// Execute the mutation pipeline
    pub async fn execute(
        endpoint: &MutationEndpoint,
        client: &HttpSourceClient,
        client_params: &HashMap<String, String>,
        client_body: Option<&StructuredValue>,
    ) -> Result<StructuredValue> {
        let mut step_results: BTreeMap<String, StructuredValue> = BTreeMap::new();
        let mut completed_step_names: Vec<String> = Vec::new();

        for step in &endpoint.steps {
            // Build step path params
            let mut step_params = client_params.clone();
            for (target_param, source_path) in &step.path_mappings {
                if let Some(val) = Self::resolve_value(source_path, client_body, client_params, &step_results) {
                    if let Some(s) = val.as_str() {
                        step_params.insert(target_param.clone(), s.to_string());
                    } else if let StructuredValue::Number(n) = val {
                        step_params.insert(target_param.clone(), n.to_string());
                    }
                }
            }

            // Build step body
            let step_body = if !step.body_mappings.is_empty() {
                let mut body_map = BTreeMap::new();
                for (target_field, source_path) in &step.body_mappings {
                    if let Some(val) = Self::resolve_value(source_path, client_body, client_params, &step_results) {
                        body_map.insert(target_field.clone(), val);
                    }
                }
                Some(StructuredValue::Object(body_map))
            } else {
                client_body.cloned()
            };

            // Execute step
            match client.execute(&step.endpoint, &step_params, step_body.as_ref()).await {
                Ok(res) => {
                    step_results.insert(step.name.clone(), res);
                    completed_step_names.push(step.name.clone());
                }
                Err(err) => {
                    // Trigger rollbacks in reverse order
                    for step_name in completed_step_names.iter().rev() {
                        if let Some(rollback) = endpoint.rollbacks.get(step_name) {
                            let mut rollback_params = client_params.clone();
                            for (target_param, source_path) in &rollback.path_mappings {
                                if let Some(val) = Self::resolve_value(
                                    source_path,
                                    client_body,
                                    client_params,
                                    &step_results,
                                ) {
                                    if let Some(s) = val.as_str() {
                                        rollback_params.insert(target_param.clone(), s.to_string());
                                    } else if let StructuredValue::Number(n) = val {
                                        rollback_params.insert(target_param.clone(), n.to_string());
                                    }
                                }
                            }
                            let _ = client.execute(&rollback.rollback_endpoint, &rollback_params, None).await;
                        }
                    }

                    return Err(OrcrustratorError::StepFailed {
                        step: step.name.clone(),
                        reason: err.to_string(),
                    });
                }
            }
        }

        // Synthesize final response
        if !endpoint.response_mappings.is_empty() {
            let mut final_map = BTreeMap::new();
            for (target_field, source_path) in &endpoint.response_mappings {
                if let Some(val) = Self::resolve_value(source_path, client_body, client_params, &step_results) {
                    final_map.insert(target_field.clone(), val);
                }
            }
            Ok(StructuredValue::Object(final_map))
        } else if let Some(last_name) = completed_step_names.last() {
            Ok(step_results.remove(last_name).unwrap_or(StructuredValue::Null))
        } else {
            Ok(StructuredValue::Object(step_results))
        }
    }
}
