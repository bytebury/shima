use shima::{
    BillingPortalSession, CheckoutSession, CreateBillingPortalSession, CreateCheckoutSession,
    CustomerId, PriceId,
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let client = shima::Client::from_env();
    let customer = CustomerId::try_from("cus_Tv7dl9n6uy86cT").unwrap();
    let session = CreateBillingPortalSession::new(customer, "https://example.com")
        .create(&client)
        .await
        .unwrap();

    dbg!(&session);
}
