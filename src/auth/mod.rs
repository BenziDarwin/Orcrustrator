pub mod basic;
pub mod bearer;
pub mod none;
pub mod oauth1;
pub mod oauth2;

pub use basic::BasicAuth;
pub use bearer::{ApiKeyAuth, ApiKeyLocation, BearerAuth};
pub use none::NoAuth;
pub use oauth1::OAuth1Auth;
pub use oauth2::OAuth2Auth;

use crate::error::Result;
use async_trait::async_trait;
use reqwest::RequestBuilder;
use std::sync::Arc;

/// Dynamic Authenticator trait for injecting credentials into HTTP requests
#[async_trait]
pub trait Authenticator: Send + Sync {
    /// Injects authentication credentials, headers, or query parameters into the request builder
    async fn apply(&self, builder: RequestBuilder) -> Result<RequestBuilder>;

    /// Returns human-readable auth type identifier
    fn auth_type(&self) -> &'static str;
}

/// High-level enum for configuring authentication strategies
#[derive(Clone)]
pub enum AuthStrategy {
    None,
    Basic(BasicAuth),
    Bearer(BearerAuth),
    ApiKey(ApiKeyAuth),
    OAuth1(OAuth1Auth),
    OAuth2(OAuth2Auth),
    Custom(Arc<dyn Authenticator>),
}

impl AuthStrategy {
    pub fn none() -> Self {
        AuthStrategy::None
    }

    pub fn basic(username: impl Into<String>, password: Option<String>) -> Self {
        AuthStrategy::Basic(BasicAuth::new(username, password))
    }

    pub fn bearer(token: impl Into<String>) -> Self {
        AuthStrategy::Bearer(BearerAuth::new(token))
    }

    pub fn api_key_header(header: impl Into<String>, key: impl Into<String>) -> Self {
        AuthStrategy::ApiKey(ApiKeyAuth::header(header, key))
    }

    pub fn api_key_query(param: impl Into<String>, key: impl Into<String>) -> Self {
        AuthStrategy::ApiKey(ApiKeyAuth::query(param, key))
    }

    pub fn oauth1(
        consumer_key: impl Into<String>,
        consumer_secret: impl Into<String>,
        token: Option<String>,
        token_secret: Option<String>,
    ) -> Self {
        AuthStrategy::OAuth1(OAuth1Auth::new(consumer_key, consumer_secret, token, token_secret))
    }

    pub fn oauth2_client_credentials(
        token_url: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        scopes: Vec<String>,
    ) -> Self {
        AuthStrategy::OAuth2(OAuth2Auth::client_credentials(
            token_url,
            client_id,
            client_secret,
            scopes,
        ))
    }

    pub fn custom(authenticator: impl Authenticator + 'static) -> Self {
        AuthStrategy::Custom(Arc::new(authenticator))
    }
}

#[async_trait]
impl Authenticator for AuthStrategy {
    async fn apply(&self, builder: RequestBuilder) -> Result<RequestBuilder> {
        match self {
            AuthStrategy::None => NoAuth.apply(builder).await,
            AuthStrategy::Basic(b) => b.apply(builder).await,
            AuthStrategy::Bearer(b) => b.apply(builder).await,
            AuthStrategy::ApiKey(a) => a.apply(builder).await,
            AuthStrategy::OAuth1(o) => o.apply(builder).await,
            AuthStrategy::OAuth2(o) => o.apply(builder).await,
            AuthStrategy::Custom(c) => c.apply(builder).await,
        }
    }

    fn auth_type(&self) -> &'static str {
        match self {
            AuthStrategy::None => "none",
            AuthStrategy::Basic(_) => "basic",
            AuthStrategy::Bearer(_) => "bearer",
            AuthStrategy::ApiKey(_) => "api_key",
            AuthStrategy::OAuth1(_) => "oauth1",
            AuthStrategy::OAuth2(_) => "oauth2",
            AuthStrategy::Custom(c) => c.auth_type(),
        }
    }
}
