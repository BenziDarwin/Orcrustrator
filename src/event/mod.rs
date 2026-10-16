pub mod frame;
pub mod transport;

pub use frame::{decode_frame, encode_frame};
pub use transport::{ChannelTransport, EventTransport};

use serde::{Deserialize, Serialize};

/// Universal event envelope conforming to the CNCF CloudEvents v1.0 standard.
///
/// Ensures seamless interop with Kafka, NATS, AWS EventBridge, Google Cloud Eventarc,
/// and OpenTelemetry while remaining lightweight and language-neutral via MessagePack.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelationalEventEnvelope {
    /// CloudEvents specification version ("1.0")
    pub specversion: String,
    /// Unique identifier for this event occurrence (UUID v4)
    pub id: String,
    /// Contextual source URI identifying the producing orchestrator/service
    pub source: String,
    /// Type of occurrence ("orcrustrator.relation.stitched", etc.)
    pub r#type: String,
    /// Timestamp of when the event happened (RFC 3339 format)
    pub time: String,
    /// Content encoding of the data field ("application/msgpack")
    pub datacontenttype: String,
    /// Logical stream or tenant identifier
    pub stream_id: String,
    /// Monotonically increasing sequence number for ordering and replay
    pub sequence: u64,
    /// Target relation identifier
    pub relation_id: String,
    /// Relational operation performed ("Created", "Updated", "Retracted", "Stitched")
    pub operation: String,
    /// MessagePack-encoded data payload
    pub data: Option<Vec<u8>>,
}

impl RelationalEventEnvelope {
    /// Create a new CloudEvents-compliant relational event envelope
    pub fn new(
        source: impl Into<String>,
        event_type: impl Into<String>,
        stream_id: impl Into<String>,
        sequence: u64,
        relation_id: impl Into<String>,
        operation: impl Into<String>,
        data: Option<Vec<u8>>,
    ) -> Self {
        Self {
            specversion: "1.0".to_string(),
            id: uuid::Uuid::new_v4().to_string(),
            source: source.into(),
            r#type: event_type.into(),
            time: chrono::Utc::now().to_rfc3339(),
            datacontenttype: "application/msgpack".to_string(),
            stream_id: stream_id.into(),
            sequence,
            relation_id: relation_id.into(),
            operation: operation.into(),
            data,
        }
    }
}
