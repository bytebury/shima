use serde::Deserialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug, Deserialize)]
pub struct CustomerId(String);

impl TryFrom<String> for CustomerId {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.trim().is_empty() {
            return Err("ID cannot be empty".into());
        }

        if !value.starts_with("cus_") {
            return Err("Invalid ID format. Customer ID must start with 'cus_'".into());
        }

        Ok(Self(value))
    }
}

impl TryFrom<&str> for CustomerId {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.trim().is_empty() {
            return Err("ID cannot be empty".into());
        }

        if !value.starts_with("cus_") {
            return Err("Invalid ID format. Customer ID must start with 'cus_'".into());
        }

        Ok(Self(value.into()))
    }
}

impl AsRef<str> for CustomerId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for CustomerId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Default)]
pub struct CreateCustomer<'a> {
    pub name: &'a str,
    pub email: &'a str,
    pub phone: Option<&'a str>,
    pub metadata: HashMap<&'a str, &'a str>,
}

impl<'a> CreateCustomer<'a> {
    pub fn new(name: &'a str, email: &'a str) -> Self {
        Self {
            name,
            email,
            phone: None,
            metadata: HashMap::new(),
        }
    }

    pub fn to_form_params(&self) -> Vec<(Cow<'a, str>, &str)> {
        let mut params = Vec::new();

        params.push((Cow::Borrowed("name"), self.name));
        params.push((Cow::Borrowed("email"), self.email));

        if let Some(phone) = self.phone {
            params.push((Cow::Borrowed("phone"), phone));
        }

        for (k, v) in self.metadata.clone() {
            params.push((Cow::Owned(format!("metadata[{}]", k)), v));
        }

        params
    }
}

#[derive(Debug, Deserialize)]
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
    ) -> Result<Customer, reqwest::Error> {
        client
            .post("/customers", customer.to_form_params())
            .await?
            .json::<Self>()
            .await
    }
}
