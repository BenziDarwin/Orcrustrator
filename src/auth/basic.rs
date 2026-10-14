use crate::auth::Authenticator;
use crate::error::Result;
use async_trait::async_trait;
use reqwest::RequestBuilder;

/// HTTP Basic Authentication strategy
#[derive(Debug, Clone)]
pub struct BasicAuth {
    username: String,
    password: Option<String>,
}

impl BasicAuth {
    pub fn new(username: impl Into<String>, password: Option<String>) -> Self {
        Self {
            username: username.into(),
            password,
        }
    }
}

#[async_trait]
impl Authenticator for BasicAuth {
    async fn apply(&self, builder: RequestBuilder) -> Result<RequestBuilder> {
        Ok(builder.basic_auth(&self.username, self.password.as_deref()))
    }

    fn auth_type(&self) -> &'static str {
        "basic"
    }
}
