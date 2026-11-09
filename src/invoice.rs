//! Stripe Invoices, mainly as delivered by `invoice.*` webhooks.
//! See <https://docs.stripe.com/api/invoices>.

use crate::{util, CustomerId, SubscriptionId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A Stripe invoice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    /// Empty for upcoming invoices, which don't exist yet.
    #[serde(default)]
    pub id: String,
    #[serde(default, deserialize_with = "util::expandable_id")]
    pub customer: Option<CustomerId>,
    #[serde(default)]
    pub customer_email: Option<String>,
    /// Present on API versions before 2025-03-31. Prefer
    /// [`Invoice::subscription_id`], which works on all versions.
    #[serde(default, deserialize_with = "util::expandable_id")]
    pub subscription: Option<SubscriptionId>,
    #[serde(default)]
    pub parent: Option<InvoiceParent>,
    /// `draft`, `open`, `paid`, `uncollectible` or `void`.
    #[serde(default)]
    pub status: Option<String>,
    /// E.g. `subscription_create`, `subscription_cycle`, `manual`.
    #[serde(default)]
    pub billing_reason: Option<String>,
    /// Amounts are in the smallest currency unit (e.g. cents).
    #[serde(default)]
    pub amount_due: i64,
    #[serde(default)]
    pub amount_paid: i64,
    #[serde(default)]
    pub amount_remaining: i64,
    #[serde(default)]
    pub currency: Option<String>,
    /// How many times payment has been attempted.
    #[serde(default)]
    pub attempt_count: u32,
    /// Unix timestamp of the next automatic payment retry, if any.
    #[serde(default)]
    pub next_payment_attempt: Option<i64>,
    /// A Stripe-hosted page where the customer can pay this invoice. Link it
    /// in "your payment failed" emails.
    #[serde(default)]
    pub hosted_invoice_url: Option<String>,
    #[serde(default)]
    pub invoice_pdf: Option<String>,
    #[serde(default)]
    pub created: Option<i64>,
    #[serde(default, deserialize_with = "util::null_default")]
    pub metadata: HashMap<String, String>,
}

/// What created the invoice (API version 2025-03-31 and later).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceParent {
    #[serde(default)]
    pub subscription_details: Option<SubscriptionDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionDetails {
    #[serde(default, deserialize_with = "util::expandable_id")]
    pub subscription: Option<SubscriptionId>,
}

impl Invoice {
    /// The subscription this invoice belongs to, on any API version.
    pub fn subscription_id(&self) -> Option<&SubscriptionId> {
        self.subscription.as_ref().or_else(|| {
            self.parent
                .as_ref()?
                .subscription_details
                .as_ref()?
                .subscription
                .as_ref()
        })
    }

    pub fn is_paid(&self) -> bool {
        self.status.as_deref() == Some("paid")
    }

    /// `true` for the invoice that starts a subscription, as opposed to a
    /// renewal.
    pub fn is_first_invoice(&self) -> bool {
        self.billing_reason.as_deref() == Some("subscription_create")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscription_id_on_old_and_new_api_versions() {
        let old: Invoice =
            serde_json::from_str(r#"{"id":"in_1","subscription":"sub_old"}"#).unwrap();
        assert_eq!(old.subscription_id().unwrap(), "sub_old");

        let new: Invoice = serde_json::from_str(
            r#"{"id":"in_1","parent":{"type":"subscription_details","subscription_details":{"subscription":"sub_new"}}}"#,
        )
        .unwrap();
        assert_eq!(new.subscription_id().unwrap(), "sub_new");
    }
}
