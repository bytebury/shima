use crate::PriceId;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[default]
    Subscription,
    Payment,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Mode::Subscription => write!(f, "subscription"),
            Mode::Payment => write!(f, "payment"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineItem(pub PriceId, pub u32);

#[derive(Debug, Default)]
pub struct LineItems(pub Vec<LineItem>);

impl Serialize for LineItems {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.0.len() * 2))?;

        for (i, item) in self.0.iter().enumerate() {
            map.serialize_entry(&format!("line_items[{}][price]", i), &item.0)?;
            map.serialize_entry(&format!("line_items[{}][quantity]", i), &item.1)?;
        }
        map.end()
    }
}

#[derive(Debug, Deserialize)]
pub struct AutomaticTax {
    enabled: bool,
    liability: Option<bool>,
}

impl Serialize for AutomaticTax {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("automatic_tax[enabled]", &self.enabled)?;

        if let Some(liability) = self.liability {
            map.serialize_entry("automatic_tax[liability]", &liability)?;
        }

        map.end()
    }
}

impl AutomaticTax {
    pub fn collect_tax() -> Self {
        Self {
            enabled: true,
            liability: None,
        }
    }
}

impl Default for AutomaticTax {
    fn default() -> Self {
        Self::collect_tax()
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BillingAddressCollection {
    #[default]
    Auto,
    Required,
}

impl Display for BillingAddressCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BillingAddressCollection::Auto => write!(f, "auto"),
            BillingAddressCollection::Required => write!(f, "required"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CustomerUpdate<'a> {
    address: &'a str,
}

impl Default for CustomerUpdate<'_> {
    fn default() -> Self {
        Self { address: "auto" }
    }
}

impl<'a> Serialize for CustomerUpdate<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("customer_update[address]", &self.address)?;
        map.end()
    }
}
