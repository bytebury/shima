use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct CancelUrl<'a>(&'a str);

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct SuccessUrl<'a>(&'a str);

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct ReturnUrl<'a>(&'a str);

impl<'a> From<&'a str> for CancelUrl<'a> {
    fn from(value: &'a str) -> Self {
        Self(value)
    }
}

impl<'a> From<&'a str> for SuccessUrl<'a> {
    fn from(value: &'a str) -> Self {
        Self(value)
    }
}

impl<'a> From<&'a str> for ReturnUrl<'a> {
    fn from(value: &'a str) -> Self {
        Self(value)
    }
}
