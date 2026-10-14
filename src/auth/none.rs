use crate::auth::Authenticator;
use crate::error::Result;
use async_trait::async_trait;
use reqwest::RequestBuilder;

/// No-op authentication strategy
#[derive(Debug, Clone, Default)]
pub struct NoAuth;

#[async_trait]
impl Authenticator for NoAuth {
    async fn apply(&self, builder: RequestBuilder) -> Result<RequestBuilder> {
        Ok(builder)
    }

    fn auth_type(&self) -> &'static str {
        "none"
    }
}
