use crate::PriceId;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// The checkout mode.
#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Recurring payments.
    #[default]
    Subscription,
    /// A one-time payment.
    Payment,
    /// Save a payment method for later without charging it.
    Setup,
}

impl Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Mode::Subscription => "subscription",
            Mode::Payment => "payment",
            Mode::Setup => "setup",
        })
    }
}

/// A price and the quantity to buy (e.g. the number of seats).
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct LineItem(pub PriceId, pub u32);

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LineItems(pub Vec<LineItem>);

/// Whether Stripe Tax calculates tax automatically.
/// Requires [Stripe Tax](https://docs.stripe.com/tax) to be set up on your account.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutomaticTax {
    pub enabled: bool,
}

impl AutomaticTax {
    pub fn collect_tax() -> Self {
        Self { enabled: true }
    }

    pub fn disabled() -> Self {
        Self { enabled: false }
    }
}

impl Default for AutomaticTax {
    fn default() -> Self {
        Self::collect_tax()
    }
}

impl From<bool> for AutomaticTax {
    fn from(enabled: bool) -> Self {
        Self { enabled }
    }
}

/// Whether to always collect the customer's billing address.
#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BillingAddressCollection {
    /// Only when needed, e.g. for tax.
    #[default]
    Auto,
    Required,
}

impl Display for BillingAddressCollection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            BillingAddressCollection::Auto => "auto",
            BillingAddressCollection::Required => "required",
        })
    }
}

/// When to ask for a payment method during checkout.
#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethodCollection {
    #[default]
    Always,
    /// Skip the card form when nothing is due today, e.g. free trials
    /// without a card on file.
    IfRequired,
}

impl Display for PaymentMethodCollection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            PaymentMethodCollection::Always => "always",
            PaymentMethodCollection::IfRequired => "if_required",
        })
    }
}

/// Whether checkout saves details the customer enters back onto an existing
/// customer. Only sent when the session has a `customer`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CustomerUpdate {
    /// Save the billing address. Needed for automatic tax.
    pub address: bool,
    /// Save the name. Needed for tax ID collection.
    pub name: bool,
}

impl Default for CustomerUpdate {
    fn default() -> Self {
        Self { address: true, name: false }
    }
}

/// The checkout session's status.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckoutStatus {
    Open,
    Complete,
    Expired,
    #[serde(other)]
    Unknown,
}

/// Whether the checkout session has been paid.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Paid,
    Unpaid,
    /// E.g. a free trial; the customer still gets access.
    NoPaymentRequired,
    #[serde(other)]
    Unknown,
}

/// The details the customer entered during checkout.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct CustomerDetails {
    pub email: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
}
