//! # Orcrustrator
//!
//! A modular, high-performance API aggregator, relational service orchestrator, and client tool.
//!
//! Features:
//! - **Multi-Auth**: Basic, Bearer, API Key, No-Auth, OAuth 1.0a, and OAuth 2.0 with in-memory caching.
//! - **Multi-Format**: In-memory sniffing and parsing of JSON, XML, and plain text into a unified dynamic AST (`StructuredValue`).
//! - **Schema Inspection**: Live runtime structural type and field inference.
//! - **Relational DAG Orchestration (GET)**: Modeling `HasOne` (1:1), `HasMany` (1:N), and `IndependentJoin` queries executed concurrently with Tokio.
//! - **Mutation Pipelines (POST, PUT, PATCH, DELETE)**: Multi-step write pipelines with parameter forwarding and compensating rollbacks (Saga pattern).
//! - **Pluggable Caching**: Built-in MemoryCache and pluggable CacheProvider trait for Redis and PostgreSQL.
//! - **CloudEvents Binary Stream**: CNCF CloudEvents v1.0 compliant length-prefixed MessagePack relational event stream.
//! - **AST Compiler**: syn-powered discovery of `relation!` declarations into definitions.toml with overrides.toml separation.

pub mod auth;
pub mod cache;
#[cfg(feature = "compiler")]
pub mod compiler;
pub mod error;
#[cfg(feature = "events")]
pub mod event;
pub mod format;
pub mod method;
pub mod mutation;
pub mod oop;
pub mod prelude;
pub mod relation;
pub mod service;
pub mod sink;
pub mod source;

// Re-export primary types at crate root
pub use cache::{CacheProvider, MemoryCache};
#[cfg(feature = "compiler")]
pub use compiler::{
    calculate_diff, merge_manifests, scan_source_directory, validate_dag, DefinitionsManifest,
    EffectiveConfiguration, ManifestDiff, OverridesManifest,
};
pub use error::{OrcrustratorError, Result};
#[cfg(feature = "events")]
pub use event::{decode_frame, encode_frame, ChannelTransport, EventTransport, RelationalEventEnvelope};
pub use format::StructuredValue;
pub use method::HttpMethod;
pub use oop::RelationshipDeclaration;
pub use service::{Service, ServiceBuilder};
pub use source::Endpoint;
