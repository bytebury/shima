use reqwest::Response;
use serde::Serialize;

pub mod billing;
pub mod checkout;
pub mod customer;
pub mod error;
pub mod macros;
pub mod value_objects;
#[cfg(feature = "webhook")]
pub mod webhook;

pub use error::*;
pub use value_objects::*;

const STRIPE_API_BASE_URL: &str = "https://api.stripe.com/v1";

/// You can generate a client directly from your environment variables.
///
/// If you have `STRIPE_SECRET_KEY` and `STRIPE_WEBHOOK_SECRET` then you can use
/// the `from_env` method to create a client. Otherwise, you can create a client
/// using the `new` method.
///
/// @example
/// ```
/// use shima::{Client};
///
/// fn main() {
///     // Through environment variables (preferred).
///     let client = Client::from_env();
///     // Using the secret directly.
///     let client = Client::new("sk_test_123");
///     // Including webhooks
///     let client = Client::new("sk_test_123").with_webhook_secret("whsec_123");
/// }
/// ```
///
#[derive(Clone)]
pub struct Client {
    stripe_secret_key: String,
    pub(crate) stripe_webhook_secret: Option<String>,
    pub(crate) http: reqwest::Client,
}

impl Client {
    /// Creates a new client with the given Stripe secret key.
    pub fn new(stripe_secret_key: &str) -> Self {
        Client {
            stripe_secret_key: stripe_secret_key.to_string(),
            stripe_webhook_secret: None,
            http: reqwest::Client::new(),
        }
    }

    /// Adds the given webhook secret to the client.
    pub fn with_webhook_secret(mut self, webhook_secret: &str) -> Self {
        self.stripe_webhook_secret = Some(webhook_secret.to_string());
        self
    }

    /// Creates a new client from environment variables. This will fail if
    /// the `STRIPE_SECRET_KEY` environment variable is not set. The
    /// `STRIPE_WEBHOOK_SECRET` environment variable is optional.
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
