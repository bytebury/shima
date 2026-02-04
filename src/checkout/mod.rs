use std::{borrow::Cow, collections::HashMap};

use crate::customer::CustomerId;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PriceId(String);

impl TryFrom<String> for PriceId {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.trim().is_empty() {
            return Err("ID cannot be empty".into());
        }

        if !value.starts_with("price_") {
            return Err("Invalid ID format. Price ID must start with 'price_'".into());
        }

        Ok(Self(value))
    }
}

#[derive(Debug, Deserialize, Default)]
pub enum Mode {
    #[default]
    Subscription,
    Payment,
}

#[derive(Debug, Deserialize)]
pub struct LineItem {
    price: PriceId,
    quantity: u32,
}

impl LineItem {
    pub fn new(price: PriceId, quantity: u32) -> Self {
        Self { price, quantity }
    }
}

#[derive(Debug, Deserialize)]
pub struct AutomaticTax {
    enabled: bool,
    liability: Option<bool>,
}

impl AutomaticTax {
    pub fn collect() -> Self {
        Self {
            enabled: true,
            liability: None,
        }
    }
}

impl Default for AutomaticTax {
    fn default() -> Self {
        Self::collect()
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BillingAddressCollection {
    #[default]
    Required,
    Optional,
}

#[derive(Debug, Deserialize, Default)]
pub struct CustomerUpdate {
    addess: Option<String>, // TODO
}

#[derive(Debug, Deserialize, Default)]
pub struct CreateCheckoutSession<'a> {
    pub customer: Option<CustomerId>,
    pub customer_email: Option<String>,
    pub line_items: Vec<LineItem>,
    pub client_reference_id: Option<String>,
    pub cancel_url: &'a str,
    pub success_url: &'a str,
    pub mode: Mode,
    pub automatic_tax: AutomaticTax,
    pub billing_address_collection: BillingAddressCollection,
    pub customer_update: Option<CustomerUpdate>,
    pub metadata: HashMap<String, String>,
}

impl<'a> CreateCheckoutSession<'a> {
    pub fn new_subscription(
        success_url: &'a str,
        cancel_url: &'a str,
        customer: CustomerId,
        price: PriceId,
    ) -> Self {
        Self {
            line_items: vec![LineItem::new(price, 1)],
            client_reference_id: None,
            cancel_url,
            success_url,
            customer: Some(customer),
            mode: Mode::Subscription,
            automatic_tax: AutomaticTax::collect(),
            billing_address_collection: BillingAddressCollection::Required,
            ..Default::default()
        }
    }

    pub fn to_form_params(&self) -> Vec<(Cow<'_, str>, &str)> {
        vec![
            (Cow::Borrowed("success_url"), self.success_url),
            (Cow::Borrowed("cancel_url"), self.cancel_url),
            (Cow::Borrowed("mode"), "subscription"),
            (
                Cow::Borrowed("line_items[0][price]"),
                "price_1SuMu8FfDg0Klu1tJXEG8gh3",
            ),
            (Cow::Borrowed("line_items[0][quantity]"), "1"),
        ]
    }
}

#[derive(Debug, Deserialize)]
pub struct CheckoutSession {
    id: String,
    url: Option<String>,
}

impl CheckoutSession {
    pub async fn create(
        client: &crate::Client,
        session: CreateCheckoutSession<'_>,
    ) -> Result<Self, reqwest::Error> {
        let res = client
            .post("/checkout/sessions", session.to_form_params())
            .await?;

        if !res.status().is_success() {
            let body = res.text().await?;
            println!("STRIPE ERROR: {}", body); // This will tell you exactly what's wrong
        }
        client
            .post("/checkout/sessions", session.to_form_params())
            .await?
            .error_for_status()?
            .json::<Self>()
            .await
    }
}
