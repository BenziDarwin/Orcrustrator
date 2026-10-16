use crate::compiler::manifest::DefinitionsManifest;
use crate::oop::RelationshipDeclaration;
use std::collections::{BTreeMap, BTreeSet};

/// Detailed diff representation between two definitions manifests
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ManifestDiff {
    pub added: Vec<RelationshipDeclaration>,
    pub removed: Vec<RelationshipDeclaration>,
    pub modified: Vec<RelationModification>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationModification {
    pub id: String,
    pub changes: Vec<FieldChange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldChange {
    pub field_name: String,
    pub old_value: String,
    pub new_value: String,
}

impl ManifestDiff {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.modified.is_empty()
    }

    /// Render human-readable diff text
    pub fn render_diff(&self) -> String {
        let mut out = String::new();

        if self.is_empty() {
            out.push_str("Manifest is up to date (no changes detected).\n");
            return out;
        }

        for add in &self.added {
            out.push_str(&format!("+ [ADD] relation \"{}\" ({} -> {})\n", add.id, add.source, add.target));
        }

        for rem in &self.removed {
            out.push_str(&format!("- [REMOVE] relation \"{}\"\n", rem.id));
        }

        for modif in &self.modified {
            out.push_str(&format!("~ [MODIFY] relation \"{}\":\n", modif.id));
            for change in &modif.changes {
                out.push_str(&format!(
                    "    {}: \"{}\" => \"{}\"\n",
                    change.field_name, change.old_value, change.new_value
                ));
            }
        }

        out
    }
}

/// Calculate the difference between an existing manifest and freshly compiled declarations
pub fn calculate_diff(
    old_manifest: &DefinitionsManifest,
    new_manifest: &DefinitionsManifest,
) -> ManifestDiff {
    let mut diff = ManifestDiff::default();

    let old_map: BTreeMap<String, &RelationshipDeclaration> = old_manifest
        .relations
        .iter()
        .map(|r| (r.id.clone(), r))
        .collect();

    let new_map: BTreeMap<String, &RelationshipDeclaration> = new_manifest
        .relations
        .iter()
        .map(|r| (r.id.clone(), r))
        .collect();

    let old_keys: BTreeSet<String> = old_map.keys().cloned().collect();
    let new_keys: BTreeSet<String> = new_map.keys().cloned().collect();

    // Check added
    for key in new_keys.difference(&old_keys) {
        if let Some(decl) = new_map.get(key) {
            diff.added.push((*decl).clone());
        }
    }

    // Check removed
    for key in old_keys.difference(&new_keys) {
        if let Some(decl) = old_map.get(key) {
            diff.removed.push((*decl).clone());
        }
    }

    // Check modified
    for key in old_keys.intersection(&new_keys) {
        let old_decl = old_map[key];
        let new_decl = new_map[key];

        let mut changes = Vec::new();

        if old_decl.source != new_decl.source {
            changes.push(FieldChange {
                field_name: "source".to_string(),
                old_value: old_decl.source.clone(),
                new_value: new_decl.source.clone(),
            });
        }
        if old_decl.target != new_decl.target {
            changes.push(FieldChange {
                field_name: "target".to_string(),
                old_value: old_decl.target.clone(),
                new_value: new_decl.target.clone(),
            });
        }
        if old_decl.relation_type != new_decl.relation_type {
            changes.push(FieldChange {
                field_name: "relation_type".to_string(),
                old_value: old_decl.relation_type.clone(),
                new_value: new_decl.relation_type.clone(),
            });
        }
        if old_decl.foreign_key != new_decl.foreign_key {
            changes.push(FieldChange {
                field_name: "foreign_key".to_string(),
                old_value: old_decl.foreign_key.clone(),
                new_value: new_decl.foreign_key.clone(),
            });
        }
        if old_decl.embed_as != new_decl.embed_as {
            changes.push(FieldChange {
                field_name: "embed_as".to_string(),
                old_value: old_decl.embed_as.clone(),
                new_value: new_decl.embed_as.clone(),
            });
        }

        if !changes.is_empty() {
            diff.modified.push(RelationModification {
                id: key.clone(),
                changes,
            });
        }
    }

    diff
}
