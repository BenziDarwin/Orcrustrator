use crate::auth::{Authenticator, AuthStrategy};
use crate::error::{OrcrustratorError, Result};
use crate::format::{DataFormat, StructuredValue};
use crate::source::endpoint::Endpoint;
use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;

/// Concurrent HTTP client for executing requests against upstream sources
#[derive(Clone)]
pub struct HttpSourceClient {
    client: Client,
    default_base_url: String,
    default_auth: AuthStrategy,
}

impl HttpSourceClient {
    pub fn new(default_base_url: impl Into<String>, default_auth: AuthStrategy) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            default_base_url: default_base_url.into(),
            default_auth,
        }
    }

    /// Fetch an endpoint with parameters and optional request body
    pub async fn execute(
        &self,
        endpoint: &Endpoint,
        params: &HashMap<String, String>,
        body: Option<&StructuredValue>,
    ) -> Result<StructuredValue> {
        let url = endpoint.resolve_url(&self.default_base_url, params)?;
        let method = endpoint.method.to_reqwest();

        let mut req_builder = self.client.request(method, &url);

        // Apply headers
        for (k, v) in &endpoint.headers {
            req_builder = req_builder.header(k, v);
        }

        // Apply query params
        for (k, v) in &endpoint.query_params {
            req_builder = req_builder.query(&[(k, v)]);
        }

        // Apply auth
        let auth = endpoint.auth.as_ref().unwrap_or(&self.default_auth);
        req_builder = auth.apply(req_builder).await?;

        // Apply body if present
        if let Some(b) = body {
            let json_body = b.to_json_value();
            req_builder = req_builder.json(&json_body);
        }

        let resp = req_builder.send().await?;
        let status = resp.status();

        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let text_body = resp.text().await?;

        if !status.is_success() {
            return Err(OrcrustratorError::ExecutionError(format!(
                "Upstream request to '{}' returned HTTP {}: {}",
                url, status, text_body
            )));
        }

        let format = DataFormat::detect(content_type.as_deref(), &text_body);
        format.parse(&text_body)
    }
}
