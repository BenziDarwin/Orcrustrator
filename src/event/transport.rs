use crate::error::Result;
use crate::event::RelationalEventEnvelope;
use async_trait::async_trait;
use tokio::sync::broadcast;

/// Pluggable transport trait for publishing relational event streams.
///
/// Keeps Orcrustrator 100% serverless by decoupling event emission from network transport.
#[async_trait]
pub trait EventTransport: Send + Sync {
    /// Publish an event to the underlying transport
    async fn publish(&self, event: &RelationalEventEnvelope) -> Result<()>;

    /// Flush any buffered outbound frames
    async fn flush(&self) -> Result<()> {
        Ok(())
    }
}

/// In-process broadcast transport using Tokio channels.
///
/// Ideal for unit testing, in-process telemetry, and feeding local GUI bridges.
pub struct ChannelTransport {
    sender: broadcast::Sender<RelationalEventEnvelope>,
}

impl ChannelTransport {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Subscribe to the live in-process event stream
    pub fn subscribe(&self) -> broadcast::Receiver<RelationalEventEnvelope> {
        self.sender.subscribe()
    }
}

#[async_trait]
impl EventTransport for ChannelTransport {
    async fn publish(&self, event: &RelationalEventEnvelope) -> Result<()> {
        // Send event to all active receivers; if no receivers, ignore error
        let _ = self.sender.send(event.clone());
        Ok(())
    }
}
