pub mod registry;

pub use registry::{MatchedRoute, ServiceRegistry};

use crate::auth::AuthStrategy;
use crate::cache::CacheProvider;
use crate::error::{OrcrustratorError, Result};
use crate::event::{EventTransport, RelationalEventEnvelope};
use crate::format::StructuredValue;
use crate::method::HttpMethod;
use crate::mutation::{MutationEndpoint, MutationPipeline};
use crate::relation::{DagExecutor, RelationalEndpoint};
use crate::sink::DataSink;
use crate::source::HttpSourceClient;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// A configured service mesh instance aggregating upstream endpoints
#[derive(Clone)]
pub struct Service {
    pub base_url: String,
    pub client: HttpSourceClient,
    pub registry: ServiceRegistry,
    pub sinks: Vec<Arc<dyn DataSink>>,
    pub cache: Option<Arc<dyn CacheProvider>>,
    pub transport: Option<Arc<dyn EventTransport>>,
    event_sequence: Arc<AtomicU64>,
}

impl Service {
    pub fn builder(base_url: impl Into<String>) -> ServiceBuilder {
        ServiceBuilder::new(base_url)
    }

    /// Process a request directly in-memory with optional caching and event streaming
    pub async fn handle_request(
        &self,
        method: HttpMethod,
        path: &str,
        body: Option<&StructuredValue>,
    ) -> Result<StructuredValue> {
        let cache_key = format!("{}:{}", method, path);

        // 1. Check cache for GET requests
        if method == HttpMethod::Get {
            if let Some(ref cache) = self.cache {
                if let Ok(Some(cached_value)) = cache.get(&cache_key).await {
                    return Ok(cached_value);
                }
            }
        }

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

        // 2. Populate cache on GET or invalidate on mutations
        if let Some(ref cache) = self.cache {
            if method == HttpMethod::Get {
                let _ = cache.set(&cache_key, &result, None).await;
            } else {
                let _ = cache.invalidate(&cache_key).await;
            }
        }

        // 3. Emit CloudEvents-compliant telemetry frame if transport is registered
        if let Some(ref transport) = self.transport {
            let seq = self.event_sequence.fetch_add(1, Ordering::Relaxed);
            let op = match method {
                HttpMethod::Get | HttpMethod::Head | HttpMethod::Options => "Stitched",
                HttpMethod::Post => "Created",
                HttpMethod::Put | HttpMethod::Patch => "Updated",
                HttpMethod::Delete => "Retracted",
            };
            let envelope = RelationalEventEnvelope::new(
                format!("orcrustrator://{}", self.base_url.trim_start_matches("https://").trim_start_matches("http://")),
                format!("orcrustrator.relation.{}", op.to_lowercase()),
                "default-stream",
                seq,
                endpoint_path.clone(),
                op,
                result.to_json_string().ok().map(String::into_bytes),
            );
            let _ = transport.publish(&envelope).await;
        }

        // 4. Notify storage sinks
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
    cache: Option<Arc<dyn CacheProvider>>,
    transport: Option<Arc<dyn EventTransport>>,
}

impl ServiceBuilder {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            auth: AuthStrategy::none(),
            registry: ServiceRegistry::new(),
            sinks: Vec::new(),
            cache: None,
            transport: None,
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

    pub fn cache(mut self, cache: impl CacheProvider + 'static) -> Self {
        self.cache = Some(Arc::new(cache));
        self
    }

    pub fn transport(mut self, transport: impl EventTransport + 'static) -> Self {
        self.transport = Some(Arc::new(transport));
        self
    }

    pub fn build(self) -> Service {
        let client = HttpSourceClient::new(&self.base_url, self.auth);
        Service {
            base_url: self.base_url,
            client,
            registry: self.registry,
            sinks: self.sinks,
            cache: self.cache,
            transport: self.transport,
            event_sequence: Arc::new(AtomicU64::new(1)),
        }
    }
}
