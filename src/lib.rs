use reqwest::Response;
use serde::Serialize;

pub mod billing;
pub mod checkout;
pub mod customer;
pub mod error;
pub mod value_objects;
pub mod webhook;

pub use error::*;
pub use value_objects::*;

const STRIPE_API_BASE_URL: &str = "https://api.stripe.com/v1";

#[derive(Clone)]
pub struct Client {
    stripe_secret_key: String,
    pub(crate) stripe_webhook_secret: Option<String>,
    pub(crate) http: reqwest::Client,
}

impl Client {
    pub fn new(stripe_secret_key: &str) -> Self {
        Client {
            stripe_secret_key: stripe_secret_key.to_string(),
            stripe_webhook_secret: None,
            http: reqwest::Client::new(),
        }
    }

    pub fn with_webhook_secret(mut self, webhook_secret: &str) -> Self {
        self.stripe_webhook_secret = Some(webhook_secret.to_string());
        self
    }

    pub fn from_env() -> Self {
        Client {
            stripe_secret_key: std::env::var("STRIPE_SECRET_KEY")
                .expect("STRIPE_SECRET_KEY environment variable is not set"),
            stripe_webhook_secret: std::env::var("STRIPE_WEBHOOK_SECRET").ok(),
            http: reqwest::Client::new(),
        }
    }

    pub(crate) async fn post(
        &self,
        endpoint: &str,
        body: &impl Serialize,
    ) -> Result<Response, reqwest::Error> {
        self.http
            .post(format!("{}{endpoint}", STRIPE_API_BASE_URL))
            .basic_auth(self.stripe_secret_key.clone(), Some(""))
            .form(&body)
            .send()
            .await
    }
}
