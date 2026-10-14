pub mod dag;
pub mod key;
pub mod stitcher;

pub use dag::DagExecutor;
pub use key::KeyExtractor;
pub use stitcher::RelationalStitcher;

use crate::source::Endpoint;

/// Type of relational link between endpoints
#[derive(Clone)]
pub enum Relationship {
    /// 1:1 Relationship (dependent fetch where parent_field provides parameter for child endpoint)
    HasOne {
        name: String,
        endpoint: Endpoint,
        foreign_param: String,
        parent_field: String,
        embed_as: String,
    },
    /// 1:N Relationship (dependent fetch where parent_field provides parameter or list of parameters)
    HasMany {
        name: String,
        endpoint: Endpoint,
        foreign_param: String,
        parent_field: String,
        embed_as: String,
    },
    /// Independent sibling join (both endpoints fetched concurrently, joined in RAM on match keys)
    IndependentJoin {
        name: String,
        endpoint: Endpoint,
        parent_match_key: String,
        child_match_key: String,
        embed_as: String,
        is_has_many: bool,
    },
}

/// A relational endpoint definition combining a root endpoint with multiple relationships
#[derive(Clone)]
pub struct RelationalEndpoint {
    pub path_pattern: String,
    pub root_endpoint: Endpoint,
    pub root_name: String,
    pub relationships: Vec<Relationship>,
}

impl RelationalEndpoint {
    pub fn new(path_pattern: impl Into<String>, root_name: impl Into<String>, root_endpoint: Endpoint) -> Self {
        Self {
            path_pattern: path_pattern.into(),
            root_name: root_name.into(),
            root_endpoint,
            relationships: Vec::new(),
        }
    }

    pub fn add_relationship(&mut self, rel: Relationship) {
        self.relationships.push(rel);
    }
}
