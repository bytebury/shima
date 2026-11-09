//! # Shima (縞)
//!
//! A lightweight Stripe client for SaaS applications: customers, checkout,
//! subscriptions, the customer portal, usage-based billing, entitlements and
//! webhooks.
//!
//! ```no_run
//! use shima::prelude::*;
//!
//! # async fn run() -> Result<(), shima::Error> {
//! let client = Client::from_env();
//!
//! let customer = create_customer!(&client, "Jane Doe", "jane@example.com")?;
//! let session = checkout!(
//!     &client,
//!     &customer.id,
//!     "price_123",
//!     "https://example.com/success",
//!     "https://example.com/cancel",
//!     trial_days = 14,
//!     allow_promotion_codes = true,
//! )?;
//! println!("Send the user to {}", session.url.unwrap_or_default());
//! # Ok(())
//! # }
//! ```

use reqwest::RequestBuilder;
use serde::de::DeserializeOwned;
use std::fmt;

pub mod billing;
pub mod checkout;
pub mod customer;
pub mod entitlements;
pub mod error;
pub(crate) mod form;
pub mod invoice;
pub mod macros;
pub mod subscription;
pub(crate) mod util;
pub mod value_objects;
#[cfg(feature = "webhook")]
pub mod webhook;

pub use error::*;
pub use value_objects::*;

/// Everything you typically need, in one import: `use shima::prelude::*;`
pub mod prelude {
    pub use crate::billing::{
        CreateCustomerPortalSession, CreateMeterEvent, CustomerPortalSession, MeterEvent,
    };
    pub use crate::checkout::{
        BillingAddressCollection, CheckoutSession, CreateCheckoutSession, Mode,
        PaymentMethodCollection,
    };
    pub use crate::customer::{CreateCustomer, Customer, UpdateCustomer};
    pub use crate::entitlements::Entitlements;
    pub use crate::invoice::Invoice;
    pub use crate::subscription::{ProrationBehavior, Subscription, SubscriptionStatus};
    #[cfg(feature = "webhook")]
    pub use crate::webhook::{Event, Listener, ShimaEvent};
    pub use crate::{
        CancelUrl, Client, CustomerId, Error, IntoId, Metadata, PriceId, ReturnUrl,
        SubscriptionId, SuccessUrl,
    };
    pub use crate::{
        checkout, checkout_payment, create_customer, delete_customer, manage_subscriptions,
        report_usage,
    };
}

const STRIPE_API_BASE_URL: &str = "https://api.stripe.com/v1";

/// The Stripe API client. It is cheap to clone, so you can store it in your
/// application state and clone it wherever you need it.
///
/// ```no_run
/// use shima::Client;
///
/// // Through environment variables (preferred). Reads `STRIPE_SECRET_KEY` and,
/// // optionally, `STRIPE_WEBHOOK_SECRET`.
/// let client = Client::from_env();
/// // Using the secret directly.
/// let client = Client::new("sk_test_123");
/// // Including webhooks.
/// let client = Client::new("sk_test_123").with_webhook_secret("whsec_123");
/// ```
#[derive(Clone)]
pub struct Client {
    stripe_secret_key: String,
    pub(crate) stripe_webhook_secret: Option<String>,
    pub(crate) http: reqwest::Client,
    base_url: String,
    api_version: Option<String>,
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("stripe_secret_key", &"[redacted]")
            .field(
                "stripe_webhook_secret",
                &self.stripe_webhook_secret.as_ref().map(|_| "[redacted]"),
            )
            .field("base_url", &self.base_url)
            .field("api_version", &self.api_version)
            .finish()
    }
}

impl Client {
    /// Creates a new client with the given Stripe secret key.
    pub fn new(stripe_secret_key: impl Into<String>) -> Self {
        Client {
            stripe_secret_key: stripe_secret_key.into(),
            stripe_webhook_secret: None,
            http: reqwest::Client::new(),
            base_url: STRIPE_API_BASE_URL.to_string(),
            api_version: None,
        }
    }

    /// Creates a new client from the `STRIPE_SECRET_KEY` environment variable,
    /// and `STRIPE_WEBHOOK_SECRET` if it is set.
    ///
    /// # Panics
    /// Panics if `STRIPE_SECRET_KEY` is not set. Use [`Client::try_from_env`]
    /// to handle that case yourself.
    pub fn from_env() -> Self {
        Self::try_from_env().unwrap_or_else(|e| panic!("{e}"))
    }

    /// Like [`Client::from_env`], but returns an error instead of panicking
    /// when `STRIPE_SECRET_KEY` is missing or empty.
    pub fn try_from_env() -> Result<Self, Error> {
        let key = std::env::var("STRIPE_SECRET_KEY")
            .ok()
            .filter(|k| !k.trim().is_empty())
            .ok_or_else(|| Error::Config("STRIPE_SECRET_KEY environment variable is not set".into()))?;

        let mut client = Client::new(key);
        client.stripe_webhook_secret = std::env::var("STRIPE_WEBHOOK_SECRET")
            .ok()
            .filter(|s| !s.trim().is_empty());
        Ok(client)
    }

    /// Adds the given webhook signing secret (`whsec_...`) to the client.
    pub fn with_webhook_secret(mut self, webhook_secret: impl Into<String>) -> Self {
        self.stripe_webhook_secret = Some(webhook_secret.into());
        self
    }

    /// Pins the Stripe API version sent with every request (the
    /// `Stripe-Version` header). Without it, your account's default version
    /// is used.
    pub fn with_api_version(mut self, version: impl Into<String>) -> Self {
        self.api_version = Some(version.into());
        self
    }

    /// Overrides the API base URL, e.g. to point at
    /// [stripe-mock](https://github.com/stripe/stripe-mock) in tests.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into().trim_end_matches('/').to_string();
        self
    }

    /// Uses your own preconfigured `reqwest::Client` (timeouts, proxies, ...).
    pub fn with_http_client(mut self, http: reqwest::Client) -> Self {
        self.http = http;
        self
    }

    /// Returns `true` if this client uses a test mode key.
    pub fn is_test_mode(&self) -> bool {
        self.stripe_secret_key.starts_with("sk_test_") || self.stripe_secret_key.starts_with("rk_test_")
    }

    fn url(&self, endpoint: &str) -> String {
        format!("{}{endpoint}", self.base_url)
    }

    pub(crate) async fn get<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        query: &[(&str, &str)],
    ) -> Result<T, Error> {
        self.execute(self.http.get(self.url(endpoint)).query(query)).await
    }

    pub(crate) async fn post<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        form: &form::Form,
    ) -> Result<T, Error> {
        self.execute(self.http.post(self.url(endpoint)).form(form.pairs()))
            .await
    }

    pub(crate) async fn delete<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T, Error> {
        self.execute(self.http.delete(self.url(endpoint))).await
    }

    async fn execute<T: DeserializeOwned>(&self, request: RequestBuilder) -> Result<T, Error> {
        let mut request = request.basic_auth(&self.stripe_secret_key, Some(""));
        if let Some(version) = &self.api_version {
            request = request.header("Stripe-Version", version);
        }

        let res = request.send().await?;
        let status = res.status();
        let body = res.text().await?;

        if status.is_success() {
            return serde_json::from_str(&body).map_err(|e| {
                Error::Internal(format!("Failed to parse Stripe response: {e}"))
            });
        }

        match serde_json::from_str::<StripeErrorResponse>(&body) {
            Ok(mut e) => {
                e.error.status = status.as_u16();
                Err(Error::Stripe(Box::new(e.error)))
            }
            Err(_) => Err(Error::Internal(format!(
                "Stripe returned HTTP {status} with an unexpected body: {body}"
            ))),
        }
    }
}
