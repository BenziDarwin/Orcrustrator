use crate::method::HttpMethod;
use crate::mutation::MutationEndpoint;
use crate::relation::RelationalEndpoint;
use std::collections::HashMap;

/// Result of matching a request against registered endpoints
pub enum MatchedRoute<'a> {
    Read(&'a RelationalEndpoint, HashMap<String, String>),
    Mutation(&'a MutationEndpoint, HashMap<String, String>),
}

/// In-memory registry of configured endpoints and services
#[derive(Default, Clone)]
pub struct ServiceRegistry {
    pub read_endpoints: Vec<RelationalEndpoint>,
    pub mutation_endpoints: Vec<MutationEndpoint>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_read(&mut self, endpoint: RelationalEndpoint) {
        self.read_endpoints.push(endpoint);
    }

    pub fn register_mutation(&mut self, endpoint: MutationEndpoint) {
        self.mutation_endpoints.push(endpoint);
    }

    /// Match an incoming method and path to a registered route
    pub fn match_route(&self, method: HttpMethod, path: &str) -> Option<MatchedRoute<'_>> {
        let clean_path = path.trim_matches('/');

        if method == HttpMethod::Get {
            for ep in &self.read_endpoints {
                if let Some(params) = Self::match_pattern(&ep.path_pattern, clean_path) {
                    return Some(MatchedRoute::Read(ep, params));
                }
            }
        } else {
            for ep in &self.mutation_endpoints {
                if ep.method == method
                    && let Some(params) = Self::match_pattern(&ep.path_pattern, clean_path)
                {
                    return Some(MatchedRoute::Mutation(ep, params));
                }
            }
        }

        None
    }

    /// Match a pattern like "/users/{id}" against a clean path "users/42"
    pub fn match_pattern(pattern: &str, clean_path: &str) -> Option<HashMap<String, String>> {
        let clean_pattern = pattern.trim_matches('/');
        let pat_segments: Vec<&str> = clean_pattern.split('/').collect();
        let path_segments: Vec<&str> = clean_path.split('/').collect();

        if pat_segments.len() != path_segments.len() {
            return None;
        }

        let mut params = HashMap::new();
        for (pat, seg) in pat_segments.iter().zip(path_segments.iter()) {
            if pat.starts_with('{') && pat.ends_with('}') {
                let key = &pat[1..pat.len() - 1];
                params.insert(key.to_string(), seg.to_string());
            } else if pat != seg {
                return None;
            }
        }

        Some(params)
    }
}
