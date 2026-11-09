use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

/// Converts strings (or IDs you already have) into a typed Stripe ID.
///
/// Every function in Shima that takes an ID accepts `&str`, `String`,
/// `&String`, the ID itself or a reference to it:
///
/// ```no_run
/// # async fn run(client: &shima::Client, user_stripe_id: String) -> Result<(), shima::Error> {
/// use shima::customer::Customer;
///
/// let customer = Customer::retrieve(client, "cus_123").await?;
/// let customer = Customer::retrieve(client, &user_stripe_id).await?;
/// let customer = Customer::retrieve(client, &customer.id).await?;
/// # Ok(())
/// # }
/// ```
pub trait IntoId<T> {
    fn into_id(self) -> Result<T, crate::Error>;
}

impl<T, U> IntoId<T> for U
where
    U: TryInto<T>,
    crate::Error: From<U::Error>,
{
    fn into_id(self) -> Result<T, crate::Error> {
        self.try_into().map_err(crate::Error::from)
    }
}

fn validate(value: &str, prefix: &str, label: &str) -> Result<(), crate::Error> {
    let value = value.trim();
    if value.is_empty() {
        return Err(crate::Error::InvalidInput(format!("{label} cannot be empty")));
    }
    if !value.starts_with(prefix) || value.len() == prefix.len() {
        return Err(crate::Error::InvalidInput(format!(
            "Invalid {label} format. {label} must start with '{prefix}', got '{value}'"
        )));
    }
    if !value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(crate::Error::InvalidInput(format!(
            "Invalid {label} format. '{value}' contains invalid characters"
        )));
    }
    Ok(())
}

macro_rules! stripe_id {
    ($(#[$meta:meta])* $name:ident, $prefix:literal, $label:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        #[cfg_attr(feature = "sqlx", derive(sqlx::Type, sqlx::FromRow))]
        #[cfg_attr(feature = "sqlx", sqlx(transparent))]
        pub struct $name(String);

        impl $name {
            /// The prefix every ID of this kind starts with.
            pub const PREFIX: &'static str = $prefix;

            /// Returns the ID as a string slice.
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Returns the inner `String`.
            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl TryFrom<&str> for $name {
            type Error = crate::Error;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                validate(value, $prefix, $label)?;
                Ok(Self(value.trim().to_string()))
            }
        }

        impl TryFrom<String> for $name {
            type Error = crate::Error;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::try_from(value.as_str())
            }
        }

        impl TryFrom<&String> for $name {
            type Error = crate::Error;

            fn try_from(value: &String) -> Result<Self, Self::Error> {
                Self::try_from(value.as_str())
            }
        }

        impl From<&$name> for $name {
            fn from(value: &$name) -> Self {
                value.clone()
            }
        }

        impl FromStr for $name {
            type Err = crate::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::try_from(s)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl PartialEq<str> for $name {
            fn eq(&self, other: &str) -> bool {
                self.0 == other
            }
        }

        impl PartialEq<&str> for $name {
            fn eq(&self, other: &&str) -> bool {
                self.0 == *other
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl crate::util::RawId for $name {
            fn from_raw(id: String) -> Self {
                Self(id)
            }
        }
    };
}

stripe_id!(
    /// A Stripe Customer ID, e.g. `cus_NffrFeUfNV2Hib`.
    CustomerId, "cus_", "CustomerID"
);
stripe_id!(
    /// A Stripe Price ID, e.g. `price_1MoBy5LkdIwHu7ixZhnattbh`.
    PriceId, "price_", "PriceID"
);
stripe_id!(
    /// A Stripe Subscription ID, e.g. `sub_1MowQVLkdIwHu7ixeRlqHVzs`.
    SubscriptionId, "sub_", "SubscriptionID"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_ids() {
        let id = CustomerId::try_from("cus_123").unwrap();
        assert_eq!(id, "cus_123");
        assert_eq!(id.to_string(), "cus_123");
        assert!(PriceId::try_from(String::from("price_abc")).is_ok());
        assert!("sub_1".parse::<SubscriptionId>().is_ok());
    }

    #[test]
    fn rejects_invalid_ids() {
        assert!(CustomerId::try_from("").is_err());
        assert!(CustomerId::try_from("cus_").is_err());
        assert!(CustomerId::try_from("price_123").is_err());
        assert!(CustomerId::try_from("cus_123/../../charges").is_err());
        assert!(matches!(
            PriceId::try_from("prod_1"),
            Err(crate::Error::InvalidInput(_))
        ));
    }

    #[test]
    fn into_id_accepts_many_forms() {
        let owned = String::from("cus_1");
        let id: CustomerId = "cus_1".into_id().unwrap();
        let _: CustomerId = (&owned).into_id().unwrap();
        let _: CustomerId = owned.clone().into_id().unwrap();
        let _: CustomerId = (&id).into_id().unwrap();
        let _: CustomerId = id.into_id().unwrap();
    }
}
