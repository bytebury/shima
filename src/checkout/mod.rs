use crate::value_objects::{customer::CustomerId, metadata::Metadata, price::PriceId};
use serde::{Deserialize, Serialize};

mod value_objects;
use value_objects::*;

#[derive(Debug, Serialize, Default)]
pub struct CreateCheckoutSession<'a> {
    pub customer: Option<CustomerId>,
    pub customer_email: Option<String>,
    pub client_reference_id: Option<String>,
    #[serde(flatten)]
    pub customer_update: CustomerUpdate<'a>,
    pub cancel_url: &'a str,
    pub success_url: &'a str,
    pub mode: Mode,
    #[serde(flatten)]
    pub automatic_tax: AutomaticTax,
    pub billing_address_collection: BillingAddressCollection,
    #[serde(flatten)]
    pub line_items: LineItems,
    #[serde(flatten)]
    pub metadata: Metadata<'a>,
}

impl<'a> CreateCheckoutSession<'a> {
    pub fn new_subscription(
        customer: CustomerId,
        price: PriceId,
        success_url: &'a str,
        cancel_url: &'a str,
    ) -> Self {
        Self {
            customer: Some(customer),
            line_items: LineItems(vec![LineItem(price, 1)]),
            success_url,
            cancel_url,
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckoutSession {
    id: String,
    url: Option<String>,
}

impl CheckoutSession {
    pub async fn create(
        client: &crate::Client,
        session: CreateCheckoutSession<'_>,
    ) -> Result<Self, reqwest::Error> {
        let res = client.post("/checkout/sessions", &session).await?;

        if !res.status().is_success() {
            let body = res.text().await?;
            println!("STRIPE ERROR: {}", body); // This will tell you exactly what's wrong
        }
        client
            .post("/checkout/sessions", &session)
            .await?
            .error_for_status()?
            .json::<Self>()
            .await
    }
}
