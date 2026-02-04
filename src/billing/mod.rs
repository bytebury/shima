use crate::customer::CustomerId;
use serde::Deserialize;
use std::borrow::Cow;

#[derive(Debug, Deserialize)]
pub struct BillingPortalSession {
    url: String,
}

impl BillingPortalSession {
    pub async fn create(
        client: &crate::Client,
        session: CreateBillingPortalSession<'_>,
    ) -> Result<Self, reqwest::Error> {
        client
            .post("/billing_portal/sessions", session.to_form_params())
            .await?
            .json::<Self>()
            .await
    }
}

pub struct CreateBillingPortalSession<'a> {
    pub customer: CustomerId,
    pub return_url: &'a str,
}

impl<'a> CreateBillingPortalSession<'a> {
    pub fn new(customer: CustomerId, return_url: &'a str) -> Self {
        CreateBillingPortalSession {
            customer,
            return_url,
        }
    }

    pub fn to_form_params(&self) -> Vec<(Cow<'a, str>, &str)> {
        vec![
            (Cow::Borrowed("customer"), &self.customer.as_ref()),
            (Cow::Borrowed("return_url"), self.return_url),
        ]
    }
}
