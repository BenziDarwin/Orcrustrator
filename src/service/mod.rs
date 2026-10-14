pub mod registry;

pub use registry::{MatchedRoute, ServiceRegistry};

use crate::auth::AuthStrategy;
use crate::error::{OrcrustratorError, Result};
use crate::format::StructuredValue;
use crate::method::HttpMethod;
use crate::mutation::{MutationEndpoint, MutationPipeline};
use crate::relation::{DagExecutor, RelationalEndpoint};
use crate::sink::DataSink;
use crate::source::HttpSourceClient;
use std::sync::Arc;

/// A configured service mesh instance aggregating upstream endpoints
#[derive(Clone)]
pub struct Service {
    pub base_url: String,
    pub client: HttpSourceClient,
    pub registry: ServiceRegistry,
    pub sinks: Vec<Arc<dyn DataSink>>,
}

impl Service {
    pub fn builder(base_url: impl Into<String>) -> ServiceBuilder {
        ServiceBuilder::new(base_url)
    }

    /// Process a request directly in-memory (usable without spinning up an HTTP server)
    pub async fn handle_request(
        &self,
        method: HttpMethod,
        path: &str,
        body: Option<&StructuredValue>,
    ) -> Result<StructuredValue> {
        let matched = self
            .registry
            .match_route(method, path)
            .ok_or_else(|| OrcrustratorError::NotFound(format!("No route matching {} {}", method, path)))?;

        let (result, params, endpoint_path) = match matched {
            MatchedRoute::Read(ep, params) => {
                let res = DagExecutor::execute(ep, &self.client, &params).await?;
                (res, params, ep.path_pattern.clone())
            }
            MatchedRoute::Mutation(ep, params) => {
                let res = MutationPipeline::execute(ep, &self.client, &params, body).await?;
                (res, params, ep.path_pattern.clone())
            }
        };

        // Notify storage sinks
        for sink in &self.sinks {
            let _ = sink.on_data(&endpoint_path, method, &params, &result).await;
        }

        Ok(result)
    }
}

/// Fluent builder for Service
pub struct ServiceBuilder {
    base_url: String,
    auth: AuthStrategy,
    registry: ServiceRegistry,
    sinks: Vec<Arc<dyn DataSink>>,
}

impl ServiceBuilder {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            auth: AuthStrategy::none(),
            registry: ServiceRegistry::new(),
            sinks: Vec::new(),
        }
    }

    pub fn auth(mut self, auth: AuthStrategy) -> Self {
        self.auth = auth;
        self
    }

    pub fn register_endpoint(mut self, endpoint: RelationalEndpoint) -> Self {
        self.registry.register_read(endpoint);
        self
    }

    pub fn register_mutation(mut self, endpoint: MutationEndpoint) -> Self {
        self.registry.register_mutation(endpoint);
        self
    }

    pub fn sink(mut self, sink: impl DataSink + 'static) -> Self {
        self.sinks.push(Arc::new(sink));
        self
    }

    pub fn build(self) -> Service {
        let client = HttpSourceClient::new(&self.base_url, self.auth);
        Service {
            base_url: self.base_url,
            client,
            registry: self.registry,
            sinks: self.sinks,
        }
    }
}
