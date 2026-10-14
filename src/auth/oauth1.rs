use crate::auth::Authenticator;
use crate::error::{OrcrustratorError, Result};
use async_trait::async_trait;
use reqwest::RequestBuilder;

/// OAuth 1.0a authentication with consumer key, secret, and optional access token
#[derive(Debug, Clone)]
pub struct OAuth1Auth {
    consumer_key: String,
    consumer_secret: String,
    token: Option<String>,
    token_secret: Option<String>,
}

impl OAuth1Auth {
    pub fn new(
        consumer_key: impl Into<String>,
        consumer_secret: impl Into<String>,
        token: Option<String>,
        token_secret: Option<String>,
    ) -> Self {
        Self {
            consumer_key: consumer_key.into(),
            consumer_secret: consumer_secret.into(),
            token,
            token_secret,
        }
    }
}

#[async_trait]
impl Authenticator for OAuth1Auth {
    #[cfg(feature = "oauth")]
    async fn apply(&self, builder: RequestBuilder) -> Result<RequestBuilder> {
        let request = builder.try_clone().ok_or_else(|| {
            OrcrustratorError::AuthError("Cannot clone request builder for OAuth1 signing".to_string())
        })?.build()?;

        let url = request.url();
        let method = request.method().as_str();

        let consumer = oauth1::Token::new(&self.consumer_key, &self.consumer_secret);
        let token = self
            .token
            .as_ref()
            .map(|t| oauth1::Token::new(t, self.token_secret.as_deref().unwrap_or("")));

        let auth_header = oauth1::authorize(
            method,
            url.as_str(),
            &consumer,
            token.as_ref(),
            None,
        );

        Ok(builder.header(reqwest::header::AUTHORIZATION, auth_header))
    }

    #[cfg(not(feature = "oauth"))]
    async fn apply(&self, _builder: RequestBuilder) -> Result<RequestBuilder> {
        Err(OrcrustratorError::AuthError(
            "OAuth1 is disabled. Enable the 'oauth' feature on orcrustrator.".to_string(),
        ))
    }

    fn auth_type(&self) -> &'static str {
        "oauth1"
    }
}
