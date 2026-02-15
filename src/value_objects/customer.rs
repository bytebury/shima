use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type, sqlx::FromRow))]
#[cfg_attr(feature = "sqlx", sqlx(transparent))]
pub struct CustomerId(String);

impl TryFrom<String> for CustomerId {
    type Error = crate::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for CustomerId {
    type Error = crate::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.trim().is_empty() {
            return Err("CustomerID cannot be empty".into());
        }

        if !value.starts_with("cus_") {
            return Err("Invalid CustomerID format. Customer ID must start with 'cus_'".into());
        }

        Ok(Self(value.into()))
    }
}

impl AsRef<str> for CustomerId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for CustomerId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
