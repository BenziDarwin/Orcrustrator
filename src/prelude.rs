//! Convenient prelude exporting the most commonly used types in Orcrustrator

pub use crate::auth::{
    ApiKeyAuth, ApiKeyLocation, AuthStrategy, Authenticator, BasicAuth, BearerAuth, NoAuth,
};

#[cfg(feature = "oauth")]
pub use crate::auth::{OAuth1Auth, OAuth2Auth};

pub use crate::cache::{CacheProvider, MemoryCache};
pub use crate::error::{OrcrustratorError, Result};
pub use crate::format::{
    DataFormat, DataType, FormatParser, SchemaInspector, SchemaNode, StructuredValue,
};
pub use crate::method::HttpMethod;
pub use crate::mutation::{
    CompensatingRollback, MutationEndpoint, MutationPipeline, MutationStep,
};
pub use crate::oop::{
    MutationEndpointBuilder, RelationalEndpointBuilder, RelationshipDeclaration, Transformer,
};
pub use crate::relation;
pub use crate::relation::{
    DagExecutor, KeyExtractor, RelationalEndpoint, RelationalStitcher, Relationship,
};
pub use crate::service::{MatchedRoute, Service, ServiceBuilder, ServiceRegistry};
pub use crate::sink::{DataSink, MemoryDataSink};
pub use crate::source::{DataSource, Endpoint, HttpSourceClient};

#[cfg(feature = "compiler")]
pub use crate::compiler::{
    calculate_diff, merge_manifests, scan_source_directory, validate_dag, DefinitionsManifest,
    EffectiveConfiguration, ManifestDiff, OverridesManifest, RelationOverride,
};

#[cfg(feature = "events")]
pub use crate::event::{
    decode_frame, encode_frame, ChannelTransport, EventTransport, RelationalEventEnvelope,
};
