use crate::{
    CancelUrl, StripeErrorResponse, SuccessUrl,
    value_objects::{customer::CustomerId, metadata::Metadata, price::PriceId},
};
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
    pub cancel_url: CancelUrl<'a>,
    pub success_url: SuccessUrl<'a>,
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
    ) -> Result<Self, crate::Error> {
        let res = client.post("/checkout/sessions", &session).await?;
        let status = res.status();

        if status.is_success() {
            return Ok(res.json::<Self>().await?);
        }

        match res.json::<StripeErrorResponse>().await {
            Ok(e) => Err(crate::Error::Stripe(e.error)),
            Err(e) => Err(crate::Error::Internal(format!(
                "Failed to parse Stripe error response: {e}"
            ))),
        }
    }
}
