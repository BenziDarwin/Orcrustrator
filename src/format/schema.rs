use crate::format::value::StructuredValue;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// Inferred schema type of a field or payload
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    Null,
    Boolean,
    Number,
    String,
    Array(Box<SchemaNode>),
    Object(BTreeMap<String, SchemaNode>),
    Mixed,
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataType::Null => write!(f, "null"),
            DataType::Boolean => write!(f, "boolean"),
            DataType::Number => write!(f, "number"),
            DataType::String => write!(f, "string"),
            DataType::Array(inner) => write!(f, "array<{}>", inner.data_type),
            DataType::Object(_) => write!(f, "object"),
            DataType::Mixed => write!(f, "mixed"),
        }
    }
}

/// A node in an inferred schema tree
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaNode {
    pub data_type: DataType,
    pub nullable: bool,
    pub description: Option<String>,
}

impl SchemaNode {
    pub fn new(data_type: DataType) -> Self {
        Self {
            data_type,
            nullable: false,
            description: None,
        }
    }
}

/// Structural schema inspector that analyzes StructuredValue in memory
pub struct SchemaInspector;

impl SchemaInspector {
    /// Infer schema from a StructuredValue
    pub fn infer(value: &StructuredValue) -> SchemaNode {
        match value {
            StructuredValue::Null => SchemaNode {
                data_type: DataType::Null,
                nullable: true,
                description: None,
            },
            StructuredValue::Bool(_) => SchemaNode::new(DataType::Boolean),
            StructuredValue::Number(_) => SchemaNode::new(DataType::Number),
            StructuredValue::String(_) => SchemaNode::new(DataType::String),
            StructuredValue::Array(items) => {
                if items.is_empty() {
                    SchemaNode::new(DataType::Array(Box::new(SchemaNode::new(DataType::Mixed))))
                } else {
                    // Combine child schemas
                    let inner_schema = Self::infer(&items[0]);
                    SchemaNode::new(DataType::Array(Box::new(inner_schema)))
                }
            }
            StructuredValue::Object(map) => {
                let mut fields = BTreeMap::new();
                for (k, v) in map {
                    fields.insert(k.clone(), Self::infer(v));
                }
                SchemaNode::new(DataType::Object(fields))
            }
        }
    }

    /// Render human-readable summary of the schema tree
    pub fn pretty_print(node: &SchemaNode, indent: usize) -> String {
        let pad = "  ".repeat(indent);
        match &node.data_type {
            DataType::Object(fields) => {
                let mut out = String::from("{\n");
                for (name, child) in fields {
                    out.push_str(&format!("{}{} : ", "  ".repeat(indent + 1), name));
                    out.push_str(&Self::pretty_print(child, indent + 1));
                    out.push('\n');
                }
                out.push_str(&format!("{}}}", pad));
                out
            }
            DataType::Array(inner) => {
                format!("[ {} ]", Self::pretty_print(inner, indent))
            }
            other => format!("{}", other),
        }
    }
}
