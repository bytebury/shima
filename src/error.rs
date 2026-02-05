use serde::Deserialize;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Stripe(StripeError),
    Network(reqwest::Error),
    Internal(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Stripe(msg) => write!(f, "Stripe API error: {}", msg.message),
            Error::Network(e) => write!(f, "Network error: {e}"),
            Error::Internal(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

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

#[derive(Debug, Deserialize)]
pub struct StripeError {
    pub message: String,
    pub code: Option<String>,
    pub param: Option<String>,
    pub r#type: String,
}

#[derive(Debug, Deserialize)]
pub struct StripeErrorResponse {
    pub error: StripeError,
}
