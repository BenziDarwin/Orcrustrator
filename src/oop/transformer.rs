use crate::format::StructuredValue;
use std::collections::BTreeMap;

/// Transforms and projects StructuredValue objects
pub struct Transformer;

impl Transformer {
    /// Project and rename fields from a source StructuredValue into a new object
    /// mappings: target_field_name -> source_dot_path
    pub fn project(source: &StructuredValue, mappings: &BTreeMap<String, String>) -> StructuredValue {
        let mut result = BTreeMap::new();
        for (target, path) in mappings {
            if let Some(val) = source.get_path(path) {
                result.insert(target.clone(), val.clone());
            }
        }
        StructuredValue::Object(result)
    }

    /// Merge two StructuredValue objects (deep merge)
    pub fn merge(a: &mut StructuredValue, b: &StructuredValue) {
        match (a, b) {
            (StructuredValue::Object(map_a), StructuredValue::Object(map_b)) => {
                for (k, v) in map_b {
                    if let Some(existing) = map_a.get_mut(k) {
                        Self::merge(existing, v);
                    } else {
                        map_a.insert(k.clone(), v.clone());
                    }
                }
            }
            (val_a, val_b) => {
                *val_a = val_b.clone();
            }
        }
    }
}
