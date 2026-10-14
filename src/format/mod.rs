pub mod parser;
pub mod schema;
pub mod value;

pub use parser::FormatParser;
pub use schema::{DataType, SchemaInspector, SchemaNode};
pub use value::StructuredValue;

use crate::error::Result;

/// Detected format of payload
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataFormat {
    Json,
    Xml,
    PlainText,
}

impl DataFormat {
    /// Detect format from Content-Type header or payload inspection
    pub fn detect(content_type: Option<&str>, body: &str) -> Self {
        if let Some(ct) = content_type {
            let lower = ct.to_lowercase();
            if lower.contains("application/json") || lower.contains("+json") {
                return DataFormat::Json;
            }
            if lower.contains("xml") {
                return DataFormat::Xml;
            }
            if lower.contains("text/plain") {
                return DataFormat::PlainText;
            }
        }

        // Sniff content
        let trimmed = body.trim_start();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            DataFormat::Json
        } else if trimmed.starts_with('<') {
            DataFormat::Xml
        } else {
            DataFormat::PlainText
        }
    }

    /// Parse a body string into StructuredValue
    pub fn parse(&self, body: &str) -> Result<StructuredValue> {
        match self {
            DataFormat::Json => FormatParser::parse_json(body),
            DataFormat::Xml => FormatParser::parse_xml(body),
            DataFormat::PlainText => FormatParser::parse_text(body),
        }
    }
}
