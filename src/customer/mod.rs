//! Stripe Customers. See <https://docs.stripe.com/api/customers>.

use crate::{form::Form, CustomerId, IntoId, List, Metadata};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};

/// Parameters for creating a customer.
///
/// ```
/// use shima::customer::CreateCustomer;
///
/// let user_id = 42;
/// let params = CreateCustomer::new("Jane Doe", "jane@example.com")
///     .add_metadata("user_id", user_id.to_string());
/// ```
#[derive(Debug, Default, Clone)]
pub struct CreateCustomer<'a> {
    pub name: Cow<'a, str>,
    pub email: Cow<'a, str>,
    pub phone: Option<Cow<'a, str>>,
    pub description: Option<Cow<'a, str>>,
    pub metadata: Metadata,
}

impl<'a> CreateCustomer<'a> {
    pub fn new(name: impl Into<Cow<'a, str>>, email: impl Into<Cow<'a, str>>) -> Self {
        Self {
            name: name.into(),
            email: email.into(),
            ..Default::default()
        }
    }

    pub fn phone(mut self, phone: impl Into<Cow<'a, str>>) -> Self {
        self.phone = Some(phone.into());
        self
    }

    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Adds all the given metadata, e.g. `.metadata([("user_id", "42")])`.
    pub fn metadata(mut self, metadata: impl Into<Metadata>) -> Self {
        self.metadata.extend(metadata.into());
        self
    }

    /// Adds a single metadata entry.
    pub fn add_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub(crate) fn to_form(&self) -> Form {
        let mut form = Form::new();
        form.push_non_empty("name", &self.name)
            .push_non_empty("email", &self.email)
            .push_opt("phone", self.phone.as_deref())
            .push_opt("description", self.description.as_deref())
            .push_metadata("metadata", &self.metadata);
        form
    }
}

/// Parameters for updating a customer. Only the fields you set are changed.
///
/// ```
/// use shima::customer::UpdateCustomer;
///
/// let params = UpdateCustomer::new().email("new@example.com");
/// ```
#[derive(Debug, Default, Clone)]
pub struct UpdateCustomer<'a> {
    pub name: Option<Cow<'a, str>>,
    pub email: Option<Cow<'a, str>>,
    pub phone: Option<Cow<'a, str>>,
    pub description: Option<Cow<'a, str>>,
    /// Entries to add or overwrite. Set a value to `""` to remove a key.
    pub metadata: Metadata,
}

impl<'a> UpdateCustomer<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn email(mut self, email: impl Into<Cow<'a, str>>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn phone(mut self, phone: impl Into<Cow<'a, str>>) -> Self {
        self.phone = Some(phone.into());
        self
    }

    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn metadata(mut self, metadata: impl Into<Metadata>) -> Self {
        self.metadata.extend(metadata.into());
        self
    }

    pub fn add_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub(crate) fn to_form(&self) -> Form {
        let mut form = Form::new();
        form.push_opt("name", self.name.as_deref())
            .push_opt("email", self.email.as_deref())
            .push_opt("phone", self.phone.as_deref())
            .push_opt("description", self.description.as_deref())
            .push_metadata("metadata", &self.metadata);
        form
    }
}

/// A postal address.
#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Address {
    pub line1: Option<String>,
    pub line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    /// Two-letter country code (ISO 3166-1 alpha-2).
    pub country: Option<String>,
}

/// A Stripe customer.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Customer {
    pub id: CustomerId,
    pub name: Option<String>,
    pub email: Option<String>,
    #[serde(default)]
    pub address: Option<Address>,
    pub description: Option<String>,
    pub phone: Option<String>,
    pub tax_exempt: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    /// Unix timestamp of when the customer was created.
    #[serde(default)]
    pub created: Option<i64>,
    /// `true` if the customer has been deleted. Stripe still returns deleted
    /// customers from [`Customer::retrieve`], with most fields empty.
    #[serde(default)]
    pub deleted: bool,
}

impl Customer {
    /// Creates a customer.
    pub async fn create(
        client: &crate::Client,
        customer: CreateCustomer<'_>,
    ) -> Result<Customer, crate::Error> {
        client.post("/customers", &customer.to_form()).await
    }

    /// Retrieves a customer by ID.
    pub async fn retrieve(
        client: &crate::Client,
        customer_id: impl IntoId<CustomerId>,
    ) -> Result<Customer, crate::Error> {
        let id = customer_id.into_id()?;
        client.get(&format!("/customers/{id}"), &[]).await
    }

    /// Updates a customer. Only the fields set in `params` change.
    pub async fn update(
        client: &crate::Client,
        customer_id: impl IntoId<CustomerId>,
        params: UpdateCustomer<'_>,
    ) -> Result<Customer, crate::Error> {
        let id = customer_id.into_id()?;
        client.post(&format!("/customers/{id}"), &params.to_form()).await
    }

    /// Permanently deletes a customer and cancels their subscriptions.
    pub async fn delete(
        client: &crate::Client,
        customer_id: impl IntoId<CustomerId>,
    ) -> Result<DeletedCustomer, crate::Error> {
        let id = customer_id.into_id()?;
        client.delete(&format!("/customers/{id}")).await
    }

    /// Finds the most recently created customer with this email address
    /// (an exact, case-sensitive match), if any.
    ///
    /// Useful for "get or create" flows when you haven't stored the customer
    /// ID yet:
    ///
    /// ```no_run
    /// # use shima::{Client, customer::*};
    /// # async fn run(client: &Client) -> Result<(), shima::Error> {
    /// let customer = match Customer::find_by_email(client, "jane@example.com").await? {
    ///     Some(customer) => customer,
    ///     None => Customer::create(client, CreateCustomer::new("Jane", "jane@example.com")).await?,
    /// };
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_by_email(
        client: &crate::Client,
        email: &str,
    ) -> Result<Option<Customer>, crate::Error> {
        let list: List<Customer> = client
            .get("/customers", &[("email", email), ("limit", "1")])
            .await?;
        Ok(list.data.into_iter().next())
    }
}

/// The response from deleting a customer.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeletedCustomer {
    pub id: CustomerId,
    pub deleted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_form_skips_empty_fields() {
        let form = CreateCustomer::new("Jane", "")
            .add_metadata("user_id", 42.to_string())
            .to_form();
        assert_eq!(form.get("name"), Some("Jane"));
        assert_eq!(form.get("email"), None);
        assert_eq!(form.get("phone"), None);
        assert_eq!(form.get("metadata[user_id]"), Some("42"));
    }

    #[test]
    fn deserializes_address_object_and_deleted_customer() {
        let customer: Customer = serde_json::from_str(
            r#"{"id":"cus_1","name":null,"email":"a@b.c","address":{"line1":"1 Main","line2":null,"city":"X","state":null,"postal_code":"123","country":"US"},"description":null,"phone":null,"tax_exempt":"none","metadata":{}}"#,
        )
        .unwrap();
        assert_eq!(customer.address.unwrap().country.as_deref(), Some("US"));

        let deleted: Customer =
            serde_json::from_str(r#"{"id":"cus_1","object":"customer","deleted":true}"#).unwrap();
        assert!(deleted.deleted);
    }
}
