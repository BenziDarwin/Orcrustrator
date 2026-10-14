use crate::format::StructuredValue;
use std::collections::BTreeMap;

/// In-memory relational stitcher that embeds child entities into parent records
pub struct RelationalStitcher;

impl RelationalStitcher {
    /// Embed a single child value into a parent StructuredValue under `embed_as`
    pub fn embed_has_one(parent: &mut StructuredValue, embed_as: &str, child: StructuredValue) {
        match parent {
            StructuredValue::Object(map) => {
                map.insert(embed_as.to_string(), child);
            }
            _ => {
                let mut map = BTreeMap::new();
                map.insert("data".to_string(), parent.clone());
                map.insert(embed_as.to_string(), child);
                *parent = StructuredValue::Object(map);
            }
        }
    }

    /// Embed a list of children into a parent StructuredValue under `embed_as`
    pub fn embed_has_many(parent: &mut StructuredValue, embed_as: &str, children: Vec<StructuredValue>) {
        let child_array = StructuredValue::Array(children);
        match parent {
            StructuredValue::Object(map) => {
                map.insert(embed_as.to_string(), child_array);
            }
            _ => {
                let mut map = BTreeMap::new();
                map.insert("data".to_string(), parent.clone());
                map.insert(embed_as.to_string(), child_array);
                *parent = StructuredValue::Object(map);
            }
        }
    }

    /// Match and join a child array into a parent array based on matching foreign keys
    pub fn join_arrays(
        parent_arr: &mut [StructuredValue],
        parent_key: &str,
        child_arr: &[StructuredValue],
        child_key: &str,
        embed_as: &str,
        is_has_many: bool,
    ) {
        // Build child lookup index
        let mut child_index: BTreeMap<String, Vec<StructuredValue>> = BTreeMap::new();
        for child in child_arr {
            if let Some(val) = child.get_path(child_key) {
                let key_str = match val {
                    StructuredValue::String(s) => s.clone(),
                    StructuredValue::Number(n) => n.to_string(),
                    _ => continue,
                };
                child_index.entry(key_str).or_default().push(child.clone());
            }
        }

        for parent in parent_arr.iter_mut() {
            if let Some(val) = parent.get_path(parent_key) {
                let key_str = match val {
                    StructuredValue::String(s) => s.clone(),
                    StructuredValue::Number(n) => n.to_string(),
                    _ => continue,
                };

                if let Some(matched) = child_index.get(&key_str) {
                    if is_has_many {
                        Self::embed_has_many(parent, embed_as, matched.clone());
                    } else if let Some(first) = matched.first() {
                        Self::embed_has_one(parent, embed_as, first.clone());
                    }
                } else if is_has_many {
                    Self::embed_has_many(parent, embed_as, Vec::new());
                } else {
                    Self::embed_has_one(parent, embed_as, StructuredValue::Null);
                }
            }
        }
    }
}
