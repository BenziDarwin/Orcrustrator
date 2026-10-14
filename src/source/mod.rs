pub mod client;
pub mod endpoint;

pub use client::HttpSourceClient;
pub use endpoint::Endpoint;

use crate::error::Result;
use crate::format::StructuredValue;
use async_trait::async_trait;
use std::collections::HashMap;

/// Abstract data source trait
#[async_trait]
pub trait DataSource: Send + Sync {
    async fn fetch(&self, params: &HashMap<String, String>) -> Result<StructuredValue>;
}
