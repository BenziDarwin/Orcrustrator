use serde::{Deserialize, Serialize};

/// Represents a declared relationship between endpoints in source code or configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationshipDeclaration {
    /// Unique identifier for the relation
    pub id: String,
    /// Root or source endpoint path
    pub source: String,
    /// Child or target endpoint path
    pub target: String,
    /// Relationship kind ("HasOne", "HasMany", "IndependentJoin")
    pub relation_type: String,
    /// Foreign key or attribute name connecting parent to child
    pub foreign_key: String,
    /// Property name to embed the child under in the stitched parent
    pub embed_as: String,
}

impl RelationshipDeclaration {
    /// Create a new relationship declaration
    pub fn new(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
        relation_type: impl Into<String>,
        foreign_key: impl Into<String>,
        embed_as: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            source: source.into(),
            target: target.into(),
            relation_type: relation_type.into(),
            foreign_key: foreign_key.into(),
            embed_as: embed_as.into(),
        }
    }
}

/// Macro for declaring machine-readable, statically-discoverable relationships in Rust source code.
///
/// # Example
/// ```rust
/// use orcrustrator::relation;
///
/// let rel = relation! {
///     id: "patient-concept-link",
///     source: "/patients/{id}",
///     target: "/concepts/{concept_id}",
///     relation_type: HasOne,
///     foreign_key: "concept_id",
///     embed_as: "concept",
/// };
/// assert_eq!(rel.id, "patient-concept-link");
/// ```
#[macro_export]
macro_rules! relation {
    (
        id: $id:expr,
        source: $source:expr,
        target: $target:expr,
        relation_type: $rel_type:ident,
        foreign_key: $fk:expr,
        embed_as: $embed:expr $(,)?
    ) => {
        $crate::oop::RelationshipDeclaration {
            id: $id.to_string(),
            source: $source.to_string(),
            target: $target.to_string(),
            relation_type: stringify!($rel_type).to_string(),
            foreign_key: $fk.to_string(),
            embed_as: $embed.to_string(),
        }
    };
}
