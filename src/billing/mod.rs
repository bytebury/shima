use crate::value_objects::customer::CustomerId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerPortalSession {
    url: String,
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

    pub async fn create(
        &self,
        client: &crate::Client,
    ) -> Result<CustomerPortalSession, reqwest::Error> {
        client
            .post("/billing_portal/sessions", self)
            .await?
            .json::<CustomerPortalSession>()
            .await
    }
}
