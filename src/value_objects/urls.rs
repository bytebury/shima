use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct CancelUrl<'a>(Cow<'a, str>);

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct SuccessUrl<'a>(Cow<'a, str>);

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct ReturnUrl<'a>(Cow<'a, str>);

impl<'a> From<&'a str> for CancelUrl<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for CancelUrl<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<&'a str> for SuccessUrl<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for SuccessUrl<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<&'a str> for ReturnUrl<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for ReturnUrl<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}
