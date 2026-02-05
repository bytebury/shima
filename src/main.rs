use shima::billing::{CreateCustomerPortalSession, CustomerPortalSession};
use shima::{
    CheckoutSession, CreateCheckoutSession, CreateCustomer, Customer, CustomerId, PriceId,
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    create_checkout_session().await.unwrap();
}

// Create a Customer in Stripe.
async fn create_customer() -> Result<Customer, shima::Error> {
    // Generate a new shima client, reading from our environment variables
    let client = shima::Client::from_env();

    // Create a new customer.
    let mut customer = CreateCustomer::new("John Doe", "john@example.com");
    customer.metadata.insert("user_id", "123456");

    // Create the customer.
    Customer::create(&client, customer).await
}

// Let customers manage their subscriptions
async fn manage_subscription() -> Result<CustomerPortalSession, shima::Error> {
    // Generate a new shima client, reading from our environment variables.
    let client = shima::Client::from_env();

    // Get the customer you want to manage.
    let customer = CustomerId::try_from("cus_123456").unwrap();

    // Setup the Customer Portal Session.
    let session = CreateCustomerPortalSession::new(customer, "https://example.com");

    // Create the Customer Portal Session.
    CustomerPortalSession::create(&client, session).await
}

// Create a Checkout Session for a Customer.
async fn create_checkout_session() -> Result<CheckoutSession, shima::Error> {
    // Generate a new shima client, reading from our environment variables
    let client = shima::Client::from_env();

    // Set your success and cancellation URLS.
    let success_url = "https://example.com/success";
    let cancel_url = "https://example.com/cancel";

    // Setup the Checkout Session.
    let mut session = CreateCheckoutSession::new_subscription(
        CustomerId::try_from("cus_1234567")?,
        PriceId::try_from("price_1234567")?,
        success_url,
        cancel_url,
    );
    session.metadata.insert("user_id", "1"); // Optional metadata

    // Create the Checkout Session.
    CheckoutSession::create(&client, session).await
}
