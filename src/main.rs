use ripe::{billing::CreateBillingPortalSession, checkout::CreateCheckoutSession};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let client = ripe::Client::new_from_env();
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
}
