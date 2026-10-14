use crate::error::{OrcrustratorError, Result};
use crate::format::value::StructuredValue;
use std::collections::BTreeMap;

/// Parses raw bytes or strings into a StructuredValue based on detected format
pub struct FormatParser;

impl FormatParser {
    /// Parse JSON string or bytes into StructuredValue
    pub fn parse_json(input: &str) -> Result<StructuredValue> {
        let val: serde_json::Value = serde_json::from_str(input)
            .map_err(|e| OrcrustratorError::FormatError(format!("Failed to parse JSON: {}", e)))?;
        Ok(StructuredValue::from_json_value(val))
    }

    /// Parse plain text into StructuredValue::String
    pub fn parse_text(input: &str) -> Result<StructuredValue> {
        Ok(StructuredValue::String(input.to_string()))
    }

    /// Parse XML into StructuredValue
    #[cfg(feature = "xml")]
    pub fn parse_xml(input: &str) -> Result<StructuredValue> {
        use quick_xml::events::Event;
        use quick_xml::reader::Reader;

        let mut reader = Reader::from_str(input);
        reader.config_mut().trim_text(true);

        #[derive(Debug)]
        struct Node {
            name: String,
            attributes: BTreeMap<String, String>,
            children: Vec<Node>,
            text: Option<String>,
        }

        impl Node {
            fn to_structured_value(&self) -> StructuredValue {
                let mut map = BTreeMap::new();

                // Add attributes prefixed with '@'
                for (k, v) in &self.attributes {
                    map.insert(format!("@{}", k), StructuredValue::String(v.clone()));
                }

                if !self.children.is_empty() {
                    let mut grouped_children: BTreeMap<String, Vec<StructuredValue>> = BTreeMap::new();
                    for child in &self.children {
                        grouped_children
                            .entry(child.name.clone())
                            .or_default()
                            .push(child.to_structured_value());
                    }

                    for (name, items) in grouped_children {
                        if items.len() == 1 {
                            map.insert(name, items.into_iter().next().unwrap());
                        } else {
                            map.insert(name, StructuredValue::Array(items));
                        }
                    }
                }

                if let Some(text) = &self.text {
                    if map.is_empty() {
                        return StructuredValue::String(text.clone());
                    } else {
                        map.insert("#text".to_string(), StructuredValue::String(text.clone()));
                    }
                }

                StructuredValue::Object(map)
            }
        }

        let mut stack: Vec<Node> = Vec::new();
        let mut root: Option<Node> = None;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let mut attributes = BTreeMap::new();
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        attributes.insert(key, value);
                    }
                    stack.push(Node {
                        name,
                        attributes,
                        children: Vec::new(),
                        text: None,
                    });
                }
                Ok(Event::End(_)) => {
                    if let Some(node) = stack.pop() {
                        if let Some(parent) = stack.last_mut() {
                            parent.children.push(node);
                        } else {
                            root = Some(node);
                        }
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let mut attributes = BTreeMap::new();
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        attributes.insert(key, value);
                    }
                    let node = Node {
                        name,
                        attributes,
                        children: Vec::new(),
                        text: None,
                    };
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(node);
                    } else {
                        root = Some(node);
                    }
                }
                Ok(Event::Text(ref e)) => {
                    let txt = e.unescape().map_err(|err| {
                        OrcrustratorError::FormatError(format!("XML unescape error: {}", err))
                    })?;
                    if !txt.trim().is_empty()
                        && let Some(curr) = stack.last_mut()
                    {
                        curr.text = Some(txt.to_string());
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(OrcrustratorError::FormatError(format!(
                        "Error reading XML at position {}: {:?}",
                        reader.error_position(),
                        e
                    )));
                }
                _ => {}
            }
            buf.clear();
        }

        if let Some(r) = root {
            let mut top = BTreeMap::new();
            top.insert(r.name.clone(), r.to_structured_value());
            Ok(StructuredValue::Object(top))
        } else {
            Ok(StructuredValue::empty_object())
        }
    }

    #[cfg(not(feature = "xml"))]
    pub fn parse_xml(_input: &str) -> Result<StructuredValue> {
        Err(OrcrustratorError::FormatError(
            "XML support is disabled. Enable the 'xml' feature on orcrustrator.".to_string(),
        ))
    }
}
