use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct PriceId(String);

impl TryFrom<String> for PriceId {
    type Error = crate::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.trim().is_empty() {
            return Err("PriceID cannot be empty".into());
        }

        if !value.starts_with("price_") {
            return Err("Invalid PriceID format. Price ID must start with 'price_'".into());
        }

        Ok(Self(value))
    }
}

impl TryFrom<&str> for PriceId {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.trim().is_empty() {
            return Err("PriceID cannot be empty".into());
        }

        if !value.starts_with("price_") {
            return Err("Invalid PriceID format. Price ID must start with 'price_'".into());
        }

        Ok(Self(value.into()))
    }
}

impl AsRef<str> for PriceId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for PriceId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
