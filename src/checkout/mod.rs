//! Stripe Checkout. See <https://docs.stripe.com/api/checkout/sessions>.

use crate::{
    form::Form, util, CancelUrl, CustomerId, Metadata, PriceId, SubscriptionId,
    SuccessUrl,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod value_objects;
pub use value_objects::*;

/// Parameters for creating a checkout session.
///
/// ```
/// use shima::checkout::*;
/// # fn run() -> Result<(), shima::Error> {
/// let session = CreateCheckoutSession::new_subscription(
///     "cus_123".try_into()?,
///     "price_123".try_into()?,
///     "https://example.com/success?session_id={CHECKOUT_SESSION_ID}".into(),
///     "https://example.com/pricing".into(),
/// )
/// .trial_days(14)
/// .allow_promotion_codes(true)
/// .quantity(5);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Default, Clone)]
pub struct CreateCheckoutSession<'a> {
    pub customer: Option<CustomerId>,
    pub customer_email: Option<String>,
    /// Your own reference, e.g. the user or team ID. It comes back in the
    /// `checkout.session.completed` webhook.
    pub client_reference_id: Option<String>,
    pub customer_update: CustomerUpdate,
    pub cancel_url: CancelUrl<'a>,
    pub success_url: SuccessUrl<'a>,
    pub mode: Mode,
    pub automatic_tax: AutomaticTax,
    pub billing_address_collection: BillingAddressCollection,
    pub line_items: LineItems,
    /// Metadata on the checkout session itself.
    pub metadata: Metadata,
    /// Lets customers enter promotion codes you created in Stripe.
    pub allow_promotion_codes: Option<bool>,
    /// Starts the subscription with a free trial of this many days.
    pub trial_period_days: Option<u32>,
    /// Metadata copied onto the subscription the session creates.
    pub subscription_metadata: Metadata,
    pub payment_method_collection: Option<PaymentMethodCollection>,
    /// Lets businesses enter their tax ID (VAT, GST, ...).
    pub tax_id_collection: Option<bool>,
}

impl<'a> CreateCheckoutSession<'a> {
    /// A subscription checkout for an existing customer.
    pub fn new_subscription(
        customer: CustomerId,
        price: PriceId,
        success_url: SuccessUrl<'a>,
        cancel_url: CancelUrl<'a>,
    ) -> Self {
        Self {
            customer: Some(customer),
            line_items: LineItems(vec![LineItem(price, 1)]),
            success_url,
            cancel_url,
            ..Default::default()
        }
    }

    /// A one-time payment checkout for an existing customer, e.g. a lifetime
    /// plan or a credit pack.
    pub fn new_payment(
        customer: CustomerId,
        price: PriceId,
        success_url: SuccessUrl<'a>,
        cancel_url: CancelUrl<'a>,
    ) -> Self {
        Self {
            mode: Mode::Payment,
            ..Self::new_subscription(customer, price, success_url, cancel_url)
        }
    }

    /// Prefills the email and lets Stripe create the customer during checkout.
    /// Clears `customer`.
    pub fn customer_email(mut self, email: impl Into<String>) -> Self {
        self.customer = None;
        self.customer_email = Some(email.into());
        self
    }

    pub fn client_reference_id(mut self, id: impl ToString) -> Self {
        self.client_reference_id = Some(id.to_string());
        self
    }

    pub fn allow_promotion_codes(mut self, allow: bool) -> Self {
        self.allow_promotion_codes = Some(allow);
        self
    }

    /// Starts subscriptions with a free trial. Ignored for one-time payments.
    pub fn trial_days(mut self, days: u32) -> Self {
        self.trial_period_days = Some(days);
        self
    }

    /// Sets the quantity of the first line item, e.g. the number of seats.
    pub fn quantity(mut self, quantity: u32) -> Self {
        if let Some(item) = self.line_items.0.first_mut() {
            item.1 = quantity;
        }
        self
    }

    /// Adds another price to the session.
    pub fn add_line_item(mut self, price: PriceId, quantity: u32) -> Self {
        self.line_items.0.push(LineItem(price, quantity));
        self
    }

    pub fn metadata(mut self, metadata: impl Into<Metadata>) -> Self {
        self.metadata.extend(metadata.into());
        self
    }

    pub fn subscription_metadata(mut self, metadata: impl Into<Metadata>) -> Self {
        self.subscription_metadata.extend(metadata.into());
        self
    }

    /// Turns Stripe Tax on or off. It is on by default.
    pub fn automatic_tax(mut self, automatic_tax: impl Into<AutomaticTax>) -> Self {
        self.automatic_tax = automatic_tax.into();
        self
    }

    pub fn billing_address_collection(mut self, collection: BillingAddressCollection) -> Self {
        self.billing_address_collection = collection;
        self
    }

    pub fn payment_method_collection(mut self, collection: PaymentMethodCollection) -> Self {
        self.payment_method_collection = Some(collection);
        self
    }

    /// Shorthand for a trial that does not ask for a card up front.
    pub fn no_card_required(self, no_card: bool) -> Self {
        if no_card {
            self.payment_method_collection(PaymentMethodCollection::IfRequired)
        } else {
            self.payment_method_collection(PaymentMethodCollection::Always)
        }
    }

    pub fn tax_id_collection(mut self, enabled: bool) -> Self {
        self.tax_id_collection = Some(enabled);
        self
    }

    pub(crate) fn to_form(&self) -> Form {
        let mut form = Form::new();
        form.push_opt("customer", self.customer.as_ref())
            .push_opt("customer_email", self.customer_email.as_deref())
            .push_opt("client_reference_id", self.client_reference_id.as_deref())
            .push("mode", self.mode)
            .push_non_empty("success_url", self.success_url.as_str())
            .push_non_empty("cancel_url", self.cancel_url.as_str())
            .push("automatic_tax[enabled]", self.automatic_tax.enabled)
            .push("billing_address_collection", self.billing_address_collection)
            .push_opt("allow_promotion_codes", self.allow_promotion_codes)
            .push_opt("tax_id_collection[enabled]", self.tax_id_collection)
            .push_metadata("metadata", &self.metadata);

        if self.customer.is_some() {
            if self.customer_update.address {
                form.push("customer_update[address]", "auto");
            }
            if self.customer_update.name || self.tax_id_collection == Some(true) {
                form.push("customer_update[name]", "auto");
            }
        }

        for (i, LineItem(price, quantity)) in self.line_items.0.iter().enumerate() {
            form.push(format!("line_items[{i}][price]"), price)
                .push(format!("line_items[{i}][quantity]"), quantity);
        }

        if self.mode == Mode::Subscription {
            form.push_opt("subscription_data[trial_period_days]", self.trial_period_days)
                .push_metadata("subscription_data[metadata]", &self.subscription_metadata);
            // Only valid for subscriptions in Stripe's API.
            form.push_opt(
                "payment_method_collection",
                self.payment_method_collection,
            );
        }

        form
    }
}

/// A Stripe checkout session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutSession {
    pub id: String,
    /// The URL to redirect your customer to. `None` once the session is
    /// complete or expired.
    pub url: Option<String>,
    #[serde(default, deserialize_with = "util::expandable_id")]
    pub customer: Option<CustomerId>,
    #[serde(default)]
    pub customer_email: Option<String>,
    #[serde(default)]
    pub customer_details: Option<CustomerDetails>,
    #[serde(default)]
    pub client_reference_id: Option<String>,
    /// The subscription created by a subscription checkout.
    #[serde(default, deserialize_with = "util::expandable_id")]
    pub subscription: Option<SubscriptionId>,
    #[serde(default)]
    pub mode: Option<Mode>,
    #[serde(default)]
    pub status: Option<CheckoutStatus>,
    #[serde(default)]
    pub payment_status: Option<PaymentStatus>,
    /// The total in the smallest currency unit (e.g. cents).
    #[serde(default)]
    pub amount_total: Option<i64>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default, deserialize_with = "util::null_default")]
    pub metadata: HashMap<String, String>,
}

impl CheckoutSession {
    /// Creates a checkout session. Redirect your customer to `session.url`.
    pub async fn create(
        client: &crate::Client,
        session: CreateCheckoutSession<'_>,
    ) -> Result<Self, crate::Error> {
        client.post("/checkout/sessions", &session.to_form()).await
    }

    /// Retrieves a checkout session, e.g. on your success page using the
    /// `{CHECKOUT_SESSION_ID}` placeholder in the success URL.
    pub async fn retrieve(client: &crate::Client, id: &str) -> Result<Self, crate::Error> {
        if !id.starts_with("cs_") || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(crate::Error::InvalidInput(format!(
                "Invalid checkout session ID '{id}'. It must start with 'cs_'"
            )));
        }
        client.get(&format!("/checkout/sessions/{id}"), &[]).await
    }

    /// `true` if the customer should get access: the session is complete and
    /// either paid or needed no payment (e.g. a free trial).
    pub fn is_fulfilled(&self) -> bool {
        self.status == Some(CheckoutStatus::Complete)
            && matches!(
                self.payment_status,
                Some(PaymentStatus::Paid | PaymentStatus::NoPaymentRequired)
            )
    }

    /// The customer's email, from whichever field Stripe filled in.
    pub fn email(&self) -> Option<&str> {
        self.customer_details
            .as_ref()
            .and_then(|d| d.email.as_deref())
            .or(self.customer_email.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> CreateCheckoutSession<'static> {
        CreateCheckoutSession::new_subscription(
            "cus_1".try_into().unwrap(),
            "price_1".try_into().unwrap(),
            "https://s".into(),
            "https://c".into(),
        )
    }

    #[test]
    fn subscription_defaults_match_previous_behavior() {
        let form = base().to_form();
        assert_eq!(form.get("customer"), Some("cus_1"));
        assert_eq!(form.get("mode"), Some("subscription"));
        assert_eq!(form.get("automatic_tax[enabled]"), Some("true"));
        assert_eq!(form.get("customer_update[address]"), Some("auto"));
        assert_eq!(form.get("billing_address_collection"), Some("auto"));
        assert_eq!(form.get("line_items[0][price]"), Some("price_1"));
        assert_eq!(form.get("line_items[0][quantity]"), Some("1"));
        assert_eq!(form.get("subscription_data[trial_period_days]"), None);
    }

    #[test]
    fn builder_options() {
        let form = base()
            .trial_days(14)
            .quantity(3)
            .allow_promotion_codes(true)
            .no_card_required(true)
            .automatic_tax(false)
            .tax_id_collection(true)
            .client_reference_id(7)
            .subscription_metadata([("team_id", "7")])
            .to_form();
        assert_eq!(form.get("subscription_data[trial_period_days]"), Some("14"));
        assert_eq!(form.get("subscription_data[metadata][team_id]"), Some("7"));
        assert_eq!(form.get("line_items[0][quantity]"), Some("3"));
        assert_eq!(form.get("allow_promotion_codes"), Some("true"));
        assert_eq!(form.get("payment_method_collection"), Some("if_required"));
        assert_eq!(form.get("automatic_tax[enabled]"), Some("false"));
        assert_eq!(form.get("customer_update[name]"), Some("auto"));
        assert_eq!(form.get("client_reference_id"), Some("7"));
    }

    #[test]
    fn customer_email_drops_customer_update() {
        let form = base().customer_email("a@b.c").to_form();
        assert_eq!(form.get("customer"), None);
        assert_eq!(form.get("customer_email"), Some("a@b.c"));
        assert_eq!(form.get("customer_update[address]"), None);
    }

    #[test]
    fn payment_mode_skips_subscription_fields() {
        let form = CreateCheckoutSession::new_payment(
            "cus_1".try_into().unwrap(),
            "price_1".try_into().unwrap(),
            "https://s".into(),
            "https://c".into(),
        )
        .trial_days(14)
        .to_form();
        assert_eq!(form.get("mode"), Some("payment"));
        assert_eq!(form.get("subscription_data[trial_period_days]"), None);
    }

    #[test]
    fn deserializes_completed_session() {
        let s: CheckoutSession = serde_json::from_str(
            r#"{"id":"cs_test_1","url":null,"customer":"cus_1","customer_details":{"email":"a@b.c"},
               "subscription":"sub_1","mode":"subscription","status":"complete",
               "payment_status":"no_payment_required","metadata":null,"client_reference_id":"42"}"#,
        )
        .unwrap();
        assert!(s.is_fulfilled());
        assert_eq!(s.email(), Some("a@b.c"));
        assert_eq!(s.subscription.unwrap(), "sub_1");
        assert!(s.metadata.is_empty());
    }
}
