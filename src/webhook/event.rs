use crate::{checkout::CheckoutSession, customer::Customer, invoice::Invoice, subscription::Subscription};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

/// A verified Stripe event, with the raw object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// The event ID (`evt_...`). Stripe may deliver an event more than once,
    /// so store processed IDs if your handler must run exactly once.
    pub id: String,
    /// E.g. `customer.subscription.updated`.
    #[serde(rename = "type")]
    pub event_type: String,
    /// Unix timestamp of when the event happened.
    #[serde(default)]
    pub created: i64,
    #[serde(default)]
    pub livemode: bool,
    #[serde(default)]
    pub api_version: Option<String>,
    pub data: EventData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    /// The object the event is about, e.g. a subscription.
    pub object: Value,
    /// For `*.updated` events, the previous values of the fields that changed.
    #[serde(default)]
    pub previous_attributes: Option<Value>,
}

impl Event {
    /// Deserializes the event's object into a type of your choice.
    pub fn object<T: DeserializeOwned>(&self) -> Result<T, crate::Error> {
        T::deserialize(&self.data.object).map_err(|e| {
            crate::Error::Webhook(format!(
                "could not parse the object of {} event {}: {e}",
                self.event_type, self.id
            ))
        })
    }

    /// Converts this event into a typed [`ShimaEvent`].
    pub fn into_shima_event(self) -> Result<ShimaEvent, crate::Error> {
        Ok(match self.event_type.as_str() {
            "checkout.session.completed" => ShimaEvent::CheckoutSessionCompleted(self.object()?),
            "checkout.session.async_payment_succeeded" => {
                ShimaEvent::CheckoutSessionAsyncPaymentSucceeded(self.object()?)
            }
            "checkout.session.expired" => ShimaEvent::CheckoutSessionExpired(self.object()?),
            "customer.subscription.created" => {
                ShimaEvent::CustomerSubscriptionCreated(self.object()?)
            }
            "customer.subscription.updated" => {
                ShimaEvent::CustomerSubscriptionUpdated(self.object()?)
            }
            "customer.subscription.deleted" => {
                ShimaEvent::CustomerSubscriptionDeleted(self.object()?)
            }
            "customer.subscription.trial_will_end" => {
                ShimaEvent::CustomerSubscriptionTrialWillEnd(self.object()?)
            }
            "customer.deleted" => ShimaEvent::CustomerDeleted(self.object()?),
            "invoice.paid" => ShimaEvent::InvoicePaid(self.object()?),
            "invoice.payment_failed" => ShimaEvent::InvoicePaymentFailed(self.object()?),
            "invoice.payment_action_required" => {
                ShimaEvent::InvoicePaymentActionRequired(self.object()?)
            }
            "invoice.upcoming" => ShimaEvent::InvoiceUpcoming(self.object()?),
            _ => ShimaEvent::Other(self),
        })
    }
}

/// The Stripe events most SaaS apps handle, with typed objects. Anything else
/// arrives as [`ShimaEvent::Other`] with the raw [`Event`].
///
/// A typical subscription lifecycle:
/// - `CheckoutSessionCompleted`: link the Stripe customer/subscription to your user.
/// - `CustomerSubscriptionUpdated`: plan changes, renewals, cancellations scheduled
///   in the portal, `past_due`... Sync `status` and the price to your database.
/// - `CustomerSubscriptionDeleted`: the subscription ended; revoke access.
/// - `InvoicePaymentFailed`: email the customer `hosted_invoice_url`.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ShimaEvent {
    /// A customer completed checkout. For subscriptions, `subscription` is set.
    /// Check [`CheckoutSession::is_fulfilled`] before granting access: delayed
    /// payment methods (e.g. bank debits) complete later with
    /// `CheckoutSessionAsyncPaymentSucceeded`.
    CheckoutSessionCompleted(CheckoutSession),
    /// A delayed payment for a checkout session succeeded.
    CheckoutSessionAsyncPaymentSucceeded(CheckoutSession),
    /// A checkout session expired without being completed (abandoned cart).
    CheckoutSessionExpired(CheckoutSession),
    CustomerSubscriptionCreated(Subscription),
    /// Any change: status, plan, quantity, scheduled cancellation, renewal...
    CustomerSubscriptionUpdated(Subscription),
    /// The subscription ended. Revoke access.
    CustomerSubscriptionDeleted(Subscription),
    /// Sent three days before a trial ends.
    CustomerSubscriptionTrialWillEnd(Subscription),
    CustomerDeleted(Customer),
    /// An invoice was paid, including every renewal.
    InvoicePaid(Invoice),
    /// A payment failed, e.g. an expired card. Stripe retries automatically.
    InvoicePaymentFailed(Invoice),
    /// The payment needs customer action, e.g. 3D Secure.
    InvoicePaymentActionRequired(Invoice),
    /// A renewal invoice is coming up.
    InvoiceUpcoming(Invoice),
    /// Any other event.
    Other(Event),
}
