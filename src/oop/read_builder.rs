use crate::relation::{RelationalEndpoint, Relationship};
use crate::source::Endpoint;

/// Fluent OOP builder for constructing RelationalEndpoint (GET) instances
pub struct RelationalEndpointBuilder {
    path_pattern: String,
    root_name: String,
    root_endpoint: Endpoint,
    relationships: Vec<Relationship>,
}

impl RelationalEndpointBuilder {
    pub fn new(path_pattern: impl Into<String>, root_name: impl Into<String>, root_endpoint: Endpoint) -> Self {
        Self {
            path_pattern: path_pattern.into(),
            root_name: root_name.into(),
            root_endpoint,
            relationships: Vec::new(),
        }
    }

    /// Add a 1:1 relationship where child endpoint is fetched using parent field value
    pub fn has_one(
        mut self,
        name: impl Into<String>,
        endpoint: Endpoint,
        foreign_param: impl Into<String>,
        parent_field: impl Into<String>,
        embed_as: impl Into<String>,
    ) -> Self {
        self.relationships.push(Relationship::HasOne {
            name: name.into(),
            endpoint,
            foreign_param: foreign_param.into(),
            parent_field: parent_field.into(),
            embed_as: embed_as.into(),
        });
        self
    }

    /// Add a 1:N relationship where child endpoint is fetched using parent field value or array of values
    pub fn has_many(
        mut self,
        name: impl Into<String>,
        endpoint: Endpoint,
        foreign_param: impl Into<String>,
        parent_field: impl Into<String>,
        embed_as: impl Into<String>,
    ) -> Self {
        self.relationships.push(Relationship::HasMany {
            name: name.into(),
            endpoint,
            foreign_param: foreign_param.into(),
            parent_field: parent_field.into(),
            embed_as: embed_as.into(),
        });
        self
    }

    /// Add an independent join where both endpoints are fetched concurrently and joined in memory
    pub fn join(
        mut self,
        name: impl Into<String>,
        endpoint: Endpoint,
        parent_match_key: impl Into<String>,
        child_match_key: impl Into<String>,
        embed_as: impl Into<String>,
        is_has_many: bool,
    ) -> Self {
        self.relationships.push(Relationship::IndependentJoin {
            name: name.into(),
            endpoint,
            parent_match_key: parent_match_key.into(),
            child_match_key: child_match_key.into(),
            embed_as: embed_as.into(),
            is_has_many,
        });
        self
    }

    /// Build the RelationalEndpoint
    pub fn build(self) -> RelationalEndpoint {
        RelationalEndpoint {
            path_pattern: self.path_pattern,
            root_name: self.root_name,
            root_endpoint: self.root_endpoint,
            relationships: self.relationships,
        }
    }
}
