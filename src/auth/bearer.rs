use crate::auth::Authenticator;
use crate::error::Result;
use async_trait::async_trait;
use reqwest::RequestBuilder;

/// Bearer token authentication
#[derive(Debug, Clone)]
pub struct BearerAuth {
    token: String,
}

impl BearerAuth {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

#[async_trait]
impl Authenticator for BearerAuth {
    async fn apply(&self, builder: RequestBuilder) -> Result<RequestBuilder> {
        Ok(builder.bearer_auth(&self.token))
    }

    fn auth_type(&self) -> &'static str {
        "bearer"
    }
}

/// Custom API Key authentication (supports header or query parameter)
#[derive(Debug, Clone)]
pub enum ApiKeyLocation {
    Header(String),
    Query(String),
}

#[derive(Debug, Clone)]
pub struct ApiKeyAuth {
    key: String,
    location: ApiKeyLocation,
}

impl ApiKeyAuth {
    pub fn header(header_name: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            location: ApiKeyLocation::Header(header_name.into()),
        }
    }

    pub fn query(param_name: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            location: ApiKeyLocation::Query(param_name.into()),
        }
    }
}

#[async_trait]
impl Authenticator for ApiKeyAuth {
    async fn apply(&self, builder: RequestBuilder) -> Result<RequestBuilder> {
        match &self.location {
            ApiKeyLocation::Header(header) => Ok(builder.header(header, &self.key)),
            ApiKeyLocation::Query(param) => Ok(builder.query(&[(param.as_str(), self.key.as_str())])),
        }
    }

    fn auth_type(&self) -> &'static str {
        "api_key"
    }
}
