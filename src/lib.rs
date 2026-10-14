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
//! - **Storage Extensibility**: Pluggable `DataSink` trait and full Serde `Serialize`/`Deserialize` support for saving to any database.

pub mod auth;
pub mod error;
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
pub use error::{OrcrustratorError, Result};
pub use format::StructuredValue;
pub use method::HttpMethod;
pub use service::{Service, ServiceBuilder};
pub use source::Endpoint;
