use crate::auth::Authenticator;
use crate::error::{OrcrustratorError, Result};
use async_trait::async_trait;
use reqwest::RequestBuilder;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Instant;

#[derive(Debug, Clone)]
struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: Option<u64>,
}

/// OAuth 2.0 Client Credentials Grant with volatile in-memory token caching
#[derive(Debug, Clone)]
pub struct OAuth2Auth {
    token_url: String,
    client_id: String,
    client_secret: String,
    scopes: Vec<String>,
    cached_token: Arc<RwLock<Option<CachedToken>>>,
}

impl OAuth2Auth {
    pub fn client_credentials(
        token_url: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        scopes: Vec<String>,
    ) -> Self {
        Self {
            token_url: token_url.into(),
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            scopes,
            cached_token: Arc::new(RwLock::new(None)),
        }
    }

    /// Retrieve valid access token, refreshing if necessary
    async fn get_valid_token(&self) -> Result<String> {
        // First read check
        {
            let read = self.cached_token.read().await;
            if let Some(token) = &*read {
                // Buffer of 30 seconds before expiration
                if token.expires_at > Instant::now() + std::time::Duration::from_secs(30) {
                    return Ok(token.access_token.clone());
                }
            }
        }

        // Acquire write lock to refresh
        let mut write = self.cached_token.write().await;
        // Double check
        if let Some(token) = &*write
            && token.expires_at > Instant::now() + std::time::Duration::from_secs(30)
        {
            return Ok(token.access_token.clone());
        }

        let client = reqwest::Client::new();
        let mut params = vec![
            ("grant_type", "client_credentials"),
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
        ];
        let scope_str = self.scopes.join(" ");
        if !self.scopes.is_empty() {
            params.push(("scope", scope_str.as_str()));
        }

        let resp = client
            .post(&self.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| OrcrustratorError::AuthError(format!("OAuth2 token request failed: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(OrcrustratorError::AuthError(format!(
                "OAuth2 token exchange returned HTTP {}: {}",
                status, body
            )));
        }

        let token_data: TokenResponse = resp.json().await.map_err(|e| {
            OrcrustratorError::AuthError(format!("Invalid OAuth2 token response JSON: {}", e))
        })?;

        let ttl = token_data.expires_in.unwrap_or(3600);
        let expires_at = Instant::now() + std::time::Duration::from_secs(ttl);

        *write = Some(CachedToken {
            access_token: token_data.access_token.clone(),
            expires_at,
        });

        Ok(token_data.access_token)
    }
}

#[async_trait]
impl Authenticator for OAuth2Auth {
    async fn apply(&self, builder: RequestBuilder) -> Result<RequestBuilder> {
        let token = self.get_valid_token().await?;
        Ok(builder.bearer_auth(token))
    }

    fn auth_type(&self) -> &'static str {
        "oauth2"
    }
}
