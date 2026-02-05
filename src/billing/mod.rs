use crate::{StripeErrorResponse, value_objects::customer::CustomerId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerPortalSession {
    url: String,
}

impl CustomerPortalSession {
    pub async fn create(
        client: &crate::Client,
        session: CreateCustomerPortalSession<'_>,
    ) -> Result<Self, crate::Error> {
        let res = client.post("/billing_portal/sessions", &session).await?;
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

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateCustomerPortalSession<'a> {
    pub customer: CustomerId,
    pub return_url: &'a str,
}

impl<'a> CreateCustomerPortalSession<'a> {
    pub fn new(customer: CustomerId, return_url: &'a str) -> Self {
        Self {
            customer,
            return_url,
        }
    }
}
