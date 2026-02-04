use std::borrow::Cow;

use reqwest::Response;

pub mod billing;
pub mod checkout;
pub mod customer;

const STRIPE_API_BASE_URL: &str = "https://api.stripe.com/v1";

pub struct Client {
    stripe_secret_key: String,
    pub(crate) http: reqwest::Client,
}

impl Client {
    pub fn new(stripe_secret_key: &str) -> Self {
        Client {
            stripe_secret_key: stripe_secret_key.to_string(),
            http: reqwest::Client::new(),
        }
    }

    pub fn from_env() -> Self {
        Client {
            stripe_secret_key: std::env::var("STRIPE_SECRET_KEY")
                .expect("STRIPE_SECRET_KEY environment variable is not set"),
            http: reqwest::Client::new(),
        }
    }

    pub(crate) async fn get(&self, endpoint: &str) -> Result<Response, reqwest::Error> {
        self.http
            .get(format!("{}{endpoint}", STRIPE_API_BASE_URL))
            .basic_auth(self.stripe_secret_key.clone(), Some(""))
            .send()
            .await
    }

    pub(crate) async fn post(
        &self,
        endpoint: &str,
        body: Vec<(Cow<'_, str>, &str)>,
    ) -> Result<Response, reqwest::Error> {
        self.http
            .post(format!("{}{endpoint}", STRIPE_API_BASE_URL))
            .basic_auth(self.stripe_secret_key.clone(), Some(""))
            .form(&body)
            .send()
            .await
    }
}
