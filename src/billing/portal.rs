use crate::{form::Form, CustomerId, IntoId, ReturnUrl};
use serde::{Deserialize, Serialize};

/// A customer portal session, where customers manage their subscription,
/// payment methods and invoices. Redirect them to `url`.
/// See <https://docs.stripe.com/customer-management>.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerPortalSession {
    #[serde(default)]
    pub id: String,
    pub url: String,
}

impl CustomerPortalSession {
    pub async fn create(
        client: &crate::Client,
        session: CreateCustomerPortalSession<'_>,
    ) -> Result<Self, crate::Error> {
        client
            .post("/billing_portal/sessions", &session.to_form())
            .await
    }
}

/// Parameters for creating a customer portal session.
#[derive(Debug, Clone)]
pub struct CreateCustomerPortalSession<'a> {
    pub customer: CustomerId,
    pub return_url: ReturnUrl<'a>,
    /// A portal configuration ID (`bpc_...`). Uses your default one if `None`.
    pub configuration: Option<String>,
}

impl<'a> CreateCustomerPortalSession<'a> {
    pub fn new(customer: CustomerId, return_url: ReturnUrl<'a>) -> Self {
        Self {
            customer,
            return_url,
            configuration: None,
        }
    }

    /// Like [`CreateCustomerPortalSession::new`], accepting any ID form.
    pub fn for_customer(
        customer: impl IntoId<CustomerId>,
        return_url: impl Into<ReturnUrl<'a>>,
    ) -> Result<Self, crate::Error> {
        Ok(Self::new(customer.into_id()?, return_url.into()))
    }

    pub fn configuration(mut self, configuration: impl Into<String>) -> Self {
        self.configuration = Some(configuration.into());
        self
    }

    pub(crate) fn to_form(&self) -> Form {
        let mut form = Form::new();
        form.push("customer", &self.customer)
            .push("return_url", &self.return_url)
            .push_opt("configuration", self.configuration.as_deref());
        form
    }
}
