use crate::error::Result;
use crate::format::StructuredValue;
use crate::relation::key::KeyExtractor;
use crate::relation::stitcher::RelationalStitcher;
use crate::relation::{RelationalEndpoint, Relationship};
use crate::source::HttpSourceClient;
use std::collections::HashMap;

/// Directed Acyclic Graph (DAG) executor for parallel and relational fetching
pub struct DagExecutor;

impl DagExecutor {
    /// Execute relational fetching for a RelationalEndpoint
    pub async fn execute(
        endpoint: &RelationalEndpoint,
        client: &HttpSourceClient,
        params: &HashMap<String, String>,
    ) -> Result<StructuredValue> {
        // Stage 0: Concurrently fetch root endpoint and any IndependentJoin endpoints
        let root_future = client.execute(&endpoint.root_endpoint, params, None);

        // Filter independent joins
        let mut independent_futures = Vec::new();
        for rel in &endpoint.relationships {
            if let Relationship::IndependentJoin { endpoint: join_ep, .. } = rel {
                independent_futures.push(client.execute(join_ep, params, None));
            }
        }

        let (root_res, join_results) = tokio::join!(
            root_future,
            futures::future::join_all(independent_futures)
        );

        let mut root_data = root_res?;

        // Apply independent joins
        let mut join_idx = 0;
        for rel in &endpoint.relationships {
            if let Relationship::IndependentJoin {
                parent_match_key,
                child_match_key,
                embed_as,
                is_has_many,
                ..
            } = rel
            {
                if let Some(Ok(child_data)) = join_results.get(join_idx) {
                    match (&mut root_data, child_data) {
                        (StructuredValue::Array(parent_arr), StructuredValue::Array(child_arr)) => {
                            RelationalStitcher::join_arrays(
                                parent_arr,
                                parent_match_key,
                                child_arr,
                                child_match_key,
                                embed_as,
                                *is_has_many,
                            );
                        }
                        (StructuredValue::Object(_), StructuredValue::Array(child_arr)) => {
                            let mut wrap = vec![root_data.clone()];
                            RelationalStitcher::join_arrays(
                                &mut wrap,
                                parent_match_key,
                                child_arr,
                                child_match_key,
                                embed_as,
                                *is_has_many,
                            );
                            if let Some(first) = wrap.into_iter().next() {
                                root_data = first;
                            }
                        }
                        _ => {
                            RelationalStitcher::embed_has_one(&mut root_data, embed_as, child_data.clone());
                        }
                    }
                }
                join_idx += 1;
            }
        }

        // Stage 1: Dependent relationships (HasOne, HasMany)
        for rel in &endpoint.relationships {
            match rel {
                Relationship::HasOne {
                    endpoint: child_ep,
                    foreign_param,
                    parent_field,
                    embed_as,
                    ..
                } => {
                    match &mut root_data {
                        StructuredValue::Object(_) => {
                            if let Some(param_val) = KeyExtractor::extract_string(&root_data, parent_field) {
                                let mut child_params = params.clone();
                                child_params.insert(foreign_param.clone(), param_val);
                                if let Ok(child_data) = client.execute(child_ep, &child_params, None).await {
                                    RelationalStitcher::embed_has_one(&mut root_data, embed_as, child_data);
                                }
                            }
                        }
                        StructuredValue::Array(arr) => {
                            // Concurrently fetch for all items in array
                            let mut tasks = Vec::new();
                            for item in arr.iter() {
                                if let Some(param_val) = KeyExtractor::extract_string(item, parent_field) {
                                    let mut child_params = params.clone();
                                    child_params.insert(foreign_param.clone(), param_val);
                                    let cl = client.clone();
                                    let ep = child_ep.clone();
                                    tasks.push(async move { cl.execute(&ep, &child_params, None).await });
                                }
                            }

                            let results = futures::future::join_all(tasks).await;
                            for (item, res) in arr.iter_mut().zip(results) {
                                if let Ok(child_data) = res {
                                    RelationalStitcher::embed_has_one(item, embed_as, child_data);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Relationship::HasMany {
                    endpoint: child_ep,
                    foreign_param,
                    parent_field,
                    embed_as,
                    ..
                } => {
                    match &mut root_data {
                        StructuredValue::Object(_) => {
                            let param_values = KeyExtractor::extract_list(&root_data, parent_field);
                            if !param_values.is_empty() {
                                let mut tasks = Vec::new();
                                for pv in param_values {
                                    let mut child_params = params.clone();
                                    child_params.insert(foreign_param.clone(), pv);
                                    let cl = client.clone();
                                    let ep = child_ep.clone();
                                    tasks.push(async move { cl.execute(&ep, &child_params, None).await });
                                }

                                let results = futures::future::join_all(tasks).await;
                                let mut children = Vec::new();
                                for c in results.into_iter().flatten() {
                                    if let StructuredValue::Array(sub_arr) = c {
                                        children.extend(sub_arr);
                                    } else {
                                        children.push(c);
                                    }
                                }
                                RelationalStitcher::embed_has_many(&mut root_data, embed_as, children);
                            } else if let Some(param_val) = KeyExtractor::extract_string(&root_data, parent_field) {
                                let mut child_params = params.clone();
                                child_params.insert(foreign_param.clone(), param_val);
                                if let Ok(child_data) = client.execute(child_ep, &child_params, None).await {
                                    if let StructuredValue::Array(items) = child_data {
                                        RelationalStitcher::embed_has_many(&mut root_data, embed_as, items);
                                    } else {
                                        RelationalStitcher::embed_has_many(&mut root_data, embed_as, vec![child_data]);
                                    }
                                }
                            }
                        }
                        StructuredValue::Array(arr) => {
                            for item in arr.iter_mut() {
                                if let Some(param_val) = KeyExtractor::extract_string(item, parent_field) {
                                    let mut child_params = params.clone();
                                    child_params.insert(foreign_param.clone(), param_val);
                                    if let Ok(child_data) = client.execute(child_ep, &child_params, None).await {
                                        if let StructuredValue::Array(items) = child_data {
                                            RelationalStitcher::embed_has_many(item, embed_as, items);
                                        } else {
                                            RelationalStitcher::embed_has_many(item, embed_as, vec![child_data]);
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Relationship::IndependentJoin { .. } => {}
            }
        }

        Ok(root_data)
    }
}
