use crate::{
    StripeErrorResponse,
    value_objects::{customer::CustomerId, metadata::Metadata},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Serialize)]
pub struct CreateCustomer<'a> {
    pub name: &'a str,
    pub email: &'a str,
    pub phone: Option<&'a str>,
    #[serde(flatten)]
    pub metadata: Metadata<'a>,
}

impl<'a> CreateCustomer<'a> {
    pub fn new(name: &'a str, email: &'a str) -> Self {
        Self {
            name,
            email,
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Customer {
    id: CustomerId,
    name: Option<String>,
    email: Option<String>,
    address: Option<String>,
    description: Option<String>,
    phone: Option<String>,
    tax_exempt: Option<String>,
    metadata: Option<HashMap<String, String>>,
}

impl Customer {
    pub async fn create(
        client: &crate::Client,
        customer: CreateCustomer<'_>,
    ) -> Result<Customer, crate::Error> {
        let res = client.post("/customers", &customer).await?;

        if res.status().is_success() {
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
