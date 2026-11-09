use crate::{form::Form, CustomerId, IntoId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reports usage for usage-based (metered) billing, e.g. API calls, tokens
/// or storage. Each event is added to the meter whose event name matches.
/// See <https://docs.stripe.com/billing/subscriptions/usage-based>.
///
/// ```no_run
/// use shima::billing::{CreateMeterEvent, MeterEvent};
/// # async fn run(client: &shima::Client) -> Result<(), shima::Error> {
/// let event = CreateMeterEvent::new("api_requests", "cus_123", 1)?
///     .identifier("req_8f3a"); // makes retries safe
/// MeterEvent::create(client, event).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct CreateMeterEvent {
    /// The meter's event name, as configured in Stripe.
    pub event_name: String,
    pub customer: CustomerId,
    pub value: u64,
    /// A unique ID for this event. Stripe ignores events whose identifier
    /// it has already seen (within 24 hours), so retries never double-bill.
    pub identifier: Option<String>,
    /// Unix timestamp of when the usage happened. Defaults to now.
    pub timestamp: Option<i64>,
    /// The payload key your meter maps to a customer.
    /// Defaults to `stripe_customer_id`.
    pub customer_key: Option<String>,
    /// The payload key your meter reads the value from. Defaults to `value`.
    pub value_key: Option<String>,
}

impl CreateMeterEvent {
    pub fn new(
        event_name: impl Into<String>,
        customer: impl IntoId<CustomerId>,
        value: u64,
    ) -> Result<Self, crate::Error> {
        Ok(Self {
            event_name: event_name.into(),
            customer: customer.into_id()?,
            value,
            identifier: None,
            timestamp: None,
            customer_key: None,
            value_key: None,
        })
    }

    pub fn identifier(mut self, identifier: impl Into<String>) -> Self {
        self.identifier = Some(identifier.into());
        self
    }

    pub fn timestamp(mut self, timestamp: i64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn customer_key(mut self, key: impl Into<String>) -> Self {
        self.customer_key = Some(key.into());
        self
    }

    pub fn value_key(mut self, key: impl Into<String>) -> Self {
        self.value_key = Some(key.into());
        self
    }

    pub(crate) fn to_form(&self) -> Form {
        let customer_key = self.customer_key.as_deref().unwrap_or("stripe_customer_id");
        let value_key = self.value_key.as_deref().unwrap_or("value");
        let mut form = Form::new();
        form.push("event_name", &self.event_name)
            .push(format!("payload[{customer_key}]"), &self.customer)
            .push(format!("payload[{value_key}]"), self.value)
            .push_opt("identifier", self.identifier.as_deref())
            .push_opt("timestamp", self.timestamp);
        form
    }
}

/// A recorded meter event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeterEvent {
    pub event_name: String,
    #[serde(default)]
    pub identifier: Option<String>,
    #[serde(default)]
    pub timestamp: Option<i64>,
    #[serde(default)]
    pub payload: HashMap<String, String>,
}

impl MeterEvent {
    pub async fn create(
        client: &crate::Client,
        event: CreateMeterEvent,
    ) -> Result<MeterEvent, crate::Error> {
        client.post("/billing/meter_events", &event.to_form()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meter_event_form() {
        let form = CreateMeterEvent::new("tokens", "cus_1", 250)
            .unwrap()
            .identifier("abc")
            .to_form();
        assert_eq!(form.get("event_name"), Some("tokens"));
        assert_eq!(form.get("payload[stripe_customer_id]"), Some("cus_1"));
        assert_eq!(form.get("payload[value]"), Some("250"));
        assert_eq!(form.get("identifier"), Some("abc"));
    }
}
