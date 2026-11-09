use serde::Deserialize;
use std::{convert::Infallible, fmt};

/// Errors returned by Shima.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Stripe rejected the request (bad parameters, card declined, not found...).
    Stripe(Box<StripeError>),
    /// The request never reached Stripe, or the connection failed.
    Network(reqwest::Error),
    /// An argument was invalid before anything was sent, e.g. a malformed ID.
    InvalidInput(String),
    /// Shima is misconfigured, e.g. a missing secret.
    Config(String),
    /// A webhook could not be verified. Respond with HTTP 400.
    Webhook(String),
    /// Anything else, such as an unexpected response body.
    Internal(String),
}

impl Error {
    /// The Stripe error, if this error came from the Stripe API.
    pub fn stripe_error(&self) -> Option<&StripeError> {
        match self {
            Error::Stripe(e) => Some(e),
            _ => None,
        }
    }

    /// `true` if Stripe answered that the object does not exist.
    pub fn is_not_found(&self) -> bool {
        self.stripe_error().is_some_and(|e| {
            e.status == 404 || e.code.as_deref() == Some("resource_missing")
        })
    }

    /// `true` if this is a card error, e.g. a declined card. Its message is
    /// safe to show to your customer.
    pub fn is_card_error(&self) -> bool {
        self.stripe_error().is_some_and(|e| e.r#type == "card_error")
    }

    /// `true` if a webhook payload failed verification.
    pub fn is_webhook_error(&self) -> bool {
        matches!(self, Error::Webhook(_))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Stripe(e) => write!(f, "Stripe API error: {}", e.message),
            Error::Network(e) => write!(f, "Network error: {e}"),
            Error::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
            Error::Config(msg) => write!(f, "Configuration error: {msg}"),
            Error::Webhook(msg) => write!(f, "Webhook error: {msg}"),
            Error::Internal(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Network(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Network(err)
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Error::Internal(msg.to_string())
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Error::Internal(msg)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Internal(err.to_string())
    }
}

impl From<Infallible> for Error {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

/// An error object returned by the Stripe API.
/// See <https://docs.stripe.com/api/errors>.
#[derive(Debug, Deserialize, Clone)]
pub struct StripeError {
    /// A human-readable message.
    #[serde(default)]
    pub message: String,
    /// A short string such as `resource_missing` or `card_declined`.
    pub code: Option<String>,
    /// The parameter the error relates to, if any.
    pub param: Option<String>,
    /// `api_error`, `card_error`, `idempotency_error` or `invalid_request_error`.
    #[serde(default)]
    pub r#type: String,
    /// For card errors, the reason the card was declined.
    #[serde(default)]
    pub decline_code: Option<String>,
    /// A link to Stripe's documentation for this error.
    #[serde(default)]
    pub doc_url: Option<String>,
    /// The HTTP status code of the response.
    #[serde(skip)]
    pub status: u16,
}

#[derive(Debug, Deserialize)]
pub struct StripeErrorResponse {
    pub error: StripeError,
}
