use serde::Deserialize;
use serde_json::Value;

/// Represents different types of events that Stripe triggers
/// and sends to your webhook endpoint. Shima only handles a subset of these events.
/// If you need to handle more events, you can extend this enum with new variants.
///
/// For audit purposes, we also include the `Unknown` variant to track events that we haven't handled yet.
pub enum ShimaEvent {
    /// A checkout session has been completed.
    /// This event is triggered when a customer completes a checkout
    /// session and payment is successful.
    CheckoutSessionCompleted(Value),
    /// A customer subscription has been deleted.
    /// This event is triggered when a customer's subscription is deleted or cancelled.
    CustomerSubscriptionDeleted(Value),
    /// An invoice payment has failed.
    /// This event is triggered when an invoice payment fails. Meaning, the customer
    /// has not paid for the invoice or subscription renewal.
    InvoicePaymentFailed(Value),
    /// Another event has occurred.
    /// This event is triggered when an event is received that we haven't handled yet.
    Other(Value),
}

#[derive(Debug, Deserialize)]
pub(crate) struct StripeEventData {
    pub object: Value,
}

#[derive(Debug, Deserialize)]
pub(crate) struct StripeEventRequest {
    pub r#type: String,
    pub data: StripeEventData,
}
