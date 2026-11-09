use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt};

macro_rules! url_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
        #[serde(transparent)]
        pub struct $name<'a>(Cow<'a, str>);

        impl $name<'_> {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl<'a> From<&'a str> for $name<'a> {
            fn from(value: &'a str) -> Self {
                Self(Cow::Borrowed(value))
            }
        }

        impl<'a> From<&'a String> for $name<'a> {
            fn from(value: &'a String) -> Self {
                Self(Cow::Borrowed(value.as_str()))
            }
        }

        impl From<String> for $name<'_> {
            fn from(value: String) -> Self {
                Self(Cow::Owned(value))
            }
        }

        impl AsRef<str> for $name<'_> {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name<'_> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

url_type!(
    /// Where Stripe sends the customer if they cancel checkout.
    CancelUrl
);
url_type!(
    /// Where Stripe sends the customer after a successful checkout. Include
    /// `{CHECKOUT_SESSION_ID}` in the URL to receive the session ID.
    SuccessUrl
);
url_type!(
    /// Where Stripe sends the customer when they leave the customer portal.
    ReturnUrl
);
