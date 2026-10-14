pub mod memory;

pub use memory::MemoryDataSink;

use crate::error::Result;
use crate::format::StructuredValue;
use crate::method::HttpMethod;
use async_trait::async_trait;
use std::collections::HashMap;

/// Pluggable storage hook trait allowing crate users to persist unified responses to any storage engine
#[async_trait]
pub trait DataSink: Send + Sync {
    /// Invoked whenever an endpoint completes an orchestration and produces data
    async fn on_data(
        &self,
        endpoint_path: &str,
        method: HttpMethod,
        params: &HashMap<String, String>,
        data: &StructuredValue,
    ) -> Result<()>;
}
