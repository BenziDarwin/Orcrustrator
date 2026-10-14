use crate::error::Result;
use crate::format::StructuredValue;
use crate::method::HttpMethod;
use crate::sink::DataSink;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Recorded entry in the MemoryDataSink
#[derive(Debug, Clone)]
pub struct RecordedEntry {
    pub endpoint_path: String,
    pub method: HttpMethod,
    pub params: HashMap<String, String>,
    pub data: StructuredValue,
}

/// In-memory implementation of DataSink for testing and debugging
#[derive(Debug, Clone, Default)]
pub struct MemoryDataSink {
    records: Arc<RwLock<Vec<RecordedEntry>>>,
}

impl MemoryDataSink {
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Retrieve all recorded entries
    pub async fn get_records(&self) -> Vec<RecordedEntry> {
        self.records.read().await.clone()
    }

    /// Clear all records
    pub async fn clear(&self) {
        self.records.write().await.clear();
    }
}

#[async_trait]
impl DataSink for MemoryDataSink {
    async fn on_data(
        &self,
        endpoint_path: &str,
        method: HttpMethod,
        params: &HashMap<String, String>,
        data: &StructuredValue,
    ) -> Result<()> {
        let entry = RecordedEntry {
            endpoint_path: endpoint_path.to_string(),
            method,
            params: params.clone(),
            data: data.clone(),
        };
        self.records.write().await.push(entry);
        Ok(())
    }
}
