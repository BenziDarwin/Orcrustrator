use thiserror::Error;

/// Result type alias for Orcrustrator operations
pub type Result<T> = std::result::Result<T, OrcrustratorError>;

/// Comprehensive error type for Orcrustrator
#[derive(Error, Debug)]
pub enum OrcrustratorError {
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("HTTP request error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Format parsing error: {0}")]
    FormatError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Missing required parameter: {0}")]
    MissingParameter(String),

    #[error("Dependency cycle detected in execution plan: {0}")]
    DependencyCycle(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Mutation step '{step}' failed: {reason}")]
    StepFailed { step: String, reason: String },

    #[error("Compensating rollback failed for step '{step}': {reason}")]
    RollbackFailed { step: String, reason: String },

    #[error("Storage sink error: {0}")]
    SinkError(String),

    #[error("Route or endpoint not found: {0}")]
    NotFound(String),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Compiler error: {0}")]
    CompilerError(String),

    #[error("Circular relation cycle detected: {0}")]
    CircularRelation(String),

    #[error("Event stream error: {0}")]
    EventStreamError(String),

    #[error("Cache error: {0}")]
    CacheError(String),
}
