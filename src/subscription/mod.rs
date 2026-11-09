//! Stripe Subscriptions. See <https://docs.stripe.com/api/subscriptions>.

use crate::{form::Form, util, CustomerId, IntoId, List, PriceId, SubscriptionId};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

/// The subscription's status.
/// See <https://docs.stripe.com/billing/subscriptions/overview#subscription-statuses>.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Incomplete,
    IncompleteExpired,
    Trialing,
    Active,
    PastDue,
    Canceled,
    Unpaid,
    Paused,
    /// A status this version of Shima doesn't know about.
    #[serde(other)]
    Unknown,
}

impl SubscriptionStatus {
    /// `true` for `active` and `trialing`: the customer should have access.
    pub fn is_active(self) -> bool {
        matches!(self, Self::Active | Self::Trialing)
    }

    /// `true` once the subscription can no longer become active again.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Canceled | Self::IncompleteExpired)
    }
}

impl fmt::Display for SubscriptionStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Incomplete => "incomplete",
            Self::IncompleteExpired => "incomplete_expired",
            Self::Trialing => "trialing",
            Self::Active => "active",
            Self::PastDue => "past_due",
            Self::Canceled => "canceled",
            Self::Unpaid => "unpaid",
            Self::Paused => "paused",
            Self::Unknown => "unknown",
        })
    }
}

/// How Stripe bills for the difference when a plan or quantity changes.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProrationBehavior {
    /// Add the prorated difference to the next invoice (Stripe's default).
    #[default]
    CreateProrations,
    /// Invoice and charge the prorated difference right away.
    AlwaysInvoice,
    /// Don't prorate; the new price applies from the next billing period.
    None,
}

impl fmt::Display for ProrationBehavior {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::CreateProrations => "create_prorations",
            Self::AlwaysInvoice => "always_invoice",
            Self::None => "none",
        })
    }
}

/// How often a recurring price bills.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recurring {
    /// `day`, `week`, `month` or `year`.
    pub interval: String,
    #[serde(default = "one")]
    pub interval_count: u32,
}

fn one() -> u32 {
    1
}

/// A price, as found on subscription items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price {
    pub id: PriceId,
    #[serde(default, deserialize_with = "util::expandable_id")]
    pub product: Option<String>,
    /// The amount in the smallest currency unit (e.g. cents).
    #[serde(default)]
    pub unit_amount: Option<i64>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub recurring: Option<Recurring>,
    /// The lookup key you set on the price, e.g. `pro_monthly`.
    #[serde(default)]
    pub lookup_key: Option<String>,
    #[serde(default)]
    pub nickname: Option<String>,
    #[serde(default, deserialize_with = "util::null_default")]
    pub metadata: HashMap<String, String>,
}

/// One price on a subscription, with its quantity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionItem {
    pub id: String,
    pub price: Price,
    #[serde(default)]
    pub quantity: Option<u64>,
    #[serde(default)]
    pub current_period_start: Option<i64>,
    #[serde(default)]
    pub current_period_end: Option<i64>,
}

/// A Stripe subscription.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: SubscriptionId,
    #[serde(deserialize_with = "util::expandable_id_required")]
    pub customer: CustomerId,
    pub status: SubscriptionStatus,
    /// `true` if the subscription will cancel at the end of the period.
    #[serde(default)]
    pub cancel_at_period_end: bool,
    #[serde(default)]
    pub cancel_at: Option<i64>,
    #[serde(default)]
    pub canceled_at: Option<i64>,
    #[serde(default)]
    pub ended_at: Option<i64>,
    #[serde(default)]
    pub trial_start: Option<i64>,
    #[serde(default)]
    pub trial_end: Option<i64>,
    /// Present on API versions before 2025-03-31. Prefer
    /// [`Subscription::current_period_end`], which works on all versions.
    #[serde(default)]
    pub current_period_start: Option<i64>,
    #[serde(default)]
    pub current_period_end: Option<i64>,
    #[serde(default)]
    pub created: Option<i64>,
    #[serde(default)]
    pub items: List<SubscriptionItem>,
    #[serde(default, deserialize_with = "util::null_default")]
    pub metadata: HashMap<String, String>,
}

impl Subscription {
    /// `true` if the customer should have access (`active` or `trialing`).
    pub fn is_active(&self) -> bool {
        self.status.is_active()
    }

    /// `true` if the subscription is in a free trial.
    pub fn is_trialing(&self) -> bool {
        self.status == SubscriptionStatus::Trialing
    }

    /// `true` if the subscription is still active but set to end, e.g. after
    /// the customer canceled in the portal.
    pub fn is_canceling(&self) -> bool {
        self.status.is_active() && (self.cancel_at_period_end || self.cancel_at.is_some())
    }

    /// The first price on the subscription. Most SaaS plans have just one.
    pub fn price(&self) -> Option<&Price> {
        self.items.data.first().map(|item| &item.price)
    }

    /// The ID of the first price on the subscription.
    pub fn price_id(&self) -> Option<&PriceId> {
        self.price().map(|p| &p.id)
    }

    /// The quantity of the first item, e.g. the number of seats.
    pub fn quantity(&self) -> Option<u64> {
        self.items.data.first().and_then(|item| item.quantity)
    }

    /// Unix timestamp of when the current billing period ends, i.e. when the
    /// subscription renews or, if canceling, when access should stop.
    pub fn current_period_end(&self) -> Option<i64> {
        self.current_period_end
            .or_else(|| self.items.data.iter().filter_map(|i| i.current_period_end).min())
    }

    /// Unix timestamp of when the current billing period started.
    pub fn current_period_start(&self) -> Option<i64> {
        self.current_period_start
            .or_else(|| self.items.data.iter().filter_map(|i| i.current_period_start).min())
    }

    /// Retrieves a subscription by ID.
    pub async fn retrieve(
        client: &crate::Client,
        id: impl IntoId<SubscriptionId>,
    ) -> Result<Subscription, crate::Error> {
        let id = id.into_id()?;
        client.get(&format!("/subscriptions/{id}"), &[]).await
    }

    /// Lists a customer's subscriptions, excluding canceled ones.
    pub async fn list_for_customer(
        client: &crate::Client,
        customer: impl IntoId<CustomerId>,
    ) -> Result<Vec<Subscription>, crate::Error> {
        let customer = customer.into_id()?;
        let list: List<Subscription> = client
            .get(
                "/subscriptions",
                &[("customer", customer.as_str()), ("limit", "100")],
            )
            .await?;
        Ok(list.data)
    }

    /// The customer's `active` or `trialing` subscription, if they have one.
    ///
    /// ```no_run
    /// # use shima::subscription::Subscription;
    /// # async fn run(client: &shima::Client) -> Result<(), shima::Error> {
    /// if let Some(sub) = Subscription::active_for_customer(client, "cus_123").await? {
    ///     println!("on {:?} until {:?}", sub.price_id(), sub.current_period_end());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn active_for_customer(
        client: &crate::Client,
        customer: impl IntoId<CustomerId>,
    ) -> Result<Option<Subscription>, crate::Error> {
        Ok(Self::list_for_customer(client, customer)
            .await?
            .into_iter()
            .find(Subscription::is_active))
    }

    /// Cancels the subscription right away. The customer loses access now.
    /// Most SaaS apps want [`Subscription::cancel_at_period_end`] instead.
    pub async fn cancel(
        client: &crate::Client,
        id: impl IntoId<SubscriptionId>,
    ) -> Result<Subscription, crate::Error> {
        let id = id.into_id()?;
        client.delete(&format!("/subscriptions/{id}")).await
    }

    /// Cancels the subscription when the current billing period ends, so
    /// the customer keeps the access they paid for.
    pub async fn cancel_at_period_end(
        client: &crate::Client,
        id: impl IntoId<SubscriptionId>,
    ) -> Result<Subscription, crate::Error> {
        Self::set_cancel_at_period_end(client, id, true).await
    }

    /// Undoes [`Subscription::cancel_at_period_end`] while the subscription
    /// is still active.
    pub async fn reactivate(
        client: &crate::Client,
        id: impl IntoId<SubscriptionId>,
    ) -> Result<Subscription, crate::Error> {
        Self::set_cancel_at_period_end(client, id, false).await
    }

    async fn set_cancel_at_period_end(
        client: &crate::Client,
        id: impl IntoId<SubscriptionId>,
        value: bool,
    ) -> Result<Subscription, crate::Error> {
        let id = id.into_id()?;
        let mut form = Form::new();
        form.push("cancel_at_period_end", value);
        client.post(&format!("/subscriptions/{id}"), &form).await
    }

    /// Switches a single-price subscription to another price, for upgrades
    /// and downgrades. Keeps the quantity.
    ///
    /// Returns [`crate::Error::InvalidInput`] if the subscription has more
    /// than one item, since it would be unclear which one to change.
    pub async fn change_price(
        client: &crate::Client,
        id: impl IntoId<SubscriptionId>,
        price: impl IntoId<PriceId>,
        proration: ProrationBehavior,
    ) -> Result<Subscription, crate::Error> {
        let price = price.into_id()?;
        let subscription = Self::retrieve(client, id).await?;
        let item = subscription.single_item()?;

        let mut form = Form::new();
        form.push("items[0][id]", &item.id)
            .push("items[0][price]", &price)
            .push("proration_behavior", proration);
        client
            .post(&format!("/subscriptions/{}", subscription.id), &form)
            .await
    }

    /// Changes the quantity of a single-price subscription, e.g. the number
    /// of seats.
    pub async fn set_quantity(
        client: &crate::Client,
        id: impl IntoId<SubscriptionId>,
        quantity: u64,
        proration: ProrationBehavior,
    ) -> Result<Subscription, crate::Error> {
        let subscription = Self::retrieve(client, id).await?;
        let item = subscription.single_item()?;

        let mut form = Form::new();
        form.push("items[0][id]", &item.id)
            .push("items[0][quantity]", quantity)
            .push("proration_behavior", proration);
        client
            .post(&format!("/subscriptions/{}", subscription.id), &form)
            .await
    }

    fn single_item(&self) -> Result<&SubscriptionItem, crate::Error> {
        match self.items.data.as_slice() {
            [item] => Ok(item),
            [] => Err(crate::Error::InvalidInput(format!(
                "Subscription {} has no items",
                self.id
            ))),
            _ => Err(crate::Error::InvalidInput(format!(
                "Subscription {} has {} items; update it directly to choose which one to change",
                self.id,
                self.items.data.len()
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NEW_API: &str = r#"{
        "id": "sub_1", "customer": "cus_1", "status": "trialing",
        "cancel_at_period_end": true, "trial_end": 1700000000, "metadata": {},
        "items": {"object": "list", "has_more": false, "data": [{
            "id": "si_1", "quantity": 3,
            "current_period_start": 1690000000, "current_period_end": 1700000000,
            "price": {"id": "price_1", "product": "prod_1", "unit_amount": 1500,
                      "currency": "usd", "recurring": {"interval": "month", "interval_count": 1},
                      "lookup_key": "pro_monthly", "metadata": {}}
        }]}
    }"#;

    #[test]
    fn deserializes_and_reads_helpers() {
        let sub: Subscription = serde_json::from_str(NEW_API).unwrap();
        assert!(sub.is_active());
        assert!(sub.is_trialing());
        assert!(sub.is_canceling());
        assert_eq!(sub.price_id().unwrap(), "price_1");
        assert_eq!(sub.quantity(), Some(3));
        assert_eq!(sub.current_period_end(), Some(1700000000));
        assert_eq!(sub.price().unwrap().lookup_key.as_deref(), Some("pro_monthly"));
        assert!(sub.single_item().is_ok());
    }

    #[test]
    fn handles_expanded_customer_and_unknown_status() {
        let sub: Subscription = serde_json::from_str(
            r#"{"id":"sub_1","customer":{"id":"cus_9","object":"customer"},"status":"brand_new"}"#,
        )
        .unwrap();
        assert_eq!(sub.customer, "cus_9");
        assert_eq!(sub.status, SubscriptionStatus::Unknown);
        assert!(!sub.is_active());
        assert!(sub.single_item().is_err());
    }
}
