use crate::auth::AuthStrategy;
use crate::error::Result;
use crate::method::HttpMethod;
use std::collections::HashMap;

/// An upstream endpoint configuration
#[derive(Clone)]
pub struct Endpoint {
    pub path_template: String,
    pub base_url: Option<String>,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub auth: Option<AuthStrategy>,
}

impl Endpoint {
    /// Create a GET endpoint
    pub fn get(path_template: impl Into<String>) -> Self {
        Self::new(HttpMethod::Get, path_template)
    }

    /// Create a POST endpoint
    pub fn post(path_template: impl Into<String>) -> Self {
        Self::new(HttpMethod::Post, path_template)
    }

    /// Create a PUT endpoint
    pub fn put(path_template: impl Into<String>) -> Self {
        Self::new(HttpMethod::Put, path_template)
    }

    /// Create a PATCH endpoint
    pub fn patch(path_template: impl Into<String>) -> Self {
        Self::new(HttpMethod::Patch, path_template)
    }

    /// Create a DELETE endpoint
    pub fn delete(path_template: impl Into<String>) -> Self {
        Self::new(HttpMethod::Delete, path_template)
    }

    /// Create endpoint with explicit method
    pub fn new(method: HttpMethod, path_template: impl Into<String>) -> Self {
        Self {
            path_template: path_template.into(),
            base_url: None,
            method,
            headers: HashMap::new(),
            query_params: HashMap::new(),
            auth: None,
        }
    }

    /// Set base URL override
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Add a request header
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Add a query parameter
    pub fn query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_params.insert(key.into(), value.into());
        self
    }

    /// Override auth strategy
    pub fn auth(mut self, auth: AuthStrategy) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Resolve effective URL by interpolating path variables e.g. `{id}`
    pub fn resolve_url(&self, default_base: &str, params: &HashMap<String, String>) -> Result<String> {
        let base = self.base_url.as_deref().unwrap_or(default_base).trim_end_matches('/');
        let mut path = self.path_template.clone();

        for (k, v) in params {
            let placeholder = format!("{{{}}}", k);
            path = path.replace(&placeholder, v);
        }

        // If path is a full URL, use it directly
        if path.starts_with("http://") || path.starts_with("https://") {
            Ok(path)
        } else {
            let clean_path = path.trim_start_matches('/');
            Ok(format!("{}/{}", base, clean_path))
        }
    }
}
