use ripe::{
    billing::CreateBillingPortalSession,
    checkout::CreateCheckoutSession,
    customer::{CreateCustomer, Customer},
};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let client = ripe::Client::from_env();
    let mut metadata = HashMap::new();
    metadata.insert("user_id", "1");

    let checkout_session = ripe::billing::BillingPortalSession::create(
        &client,
        CreateBillingPortalSession {
            customer: "cus_TulYvXsiEecISc".try_into().unwrap(),
            return_url: "https://example.com/checkout",
        },
    )
    .await;

    dbg!(&checkout_session);

    let ripe_client = ripe::Client::from_env();

    // Create a customer request struct
    let mut customer = CreateCustomer::new("John Doe", "john.doe@example.com");
    customer.metadata.insert("user_id", "1");

    // Attempt to create the customer
    Customer::create(&client, customer).await;
}
