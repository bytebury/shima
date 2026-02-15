<div align="center">
  <img src="https://raw.githubusercontent.com/bytebury/asset-manager/refs/heads/main/zebra.svg" alt="ripe logo" width="128" />
  <h1>Shima (縞)</h1>
  <p>
    <strong>Shima</strong> is a lightweight, high-performance Stripe API client library written in Rust.
The name comes from the Japanese word <strong>Shima (縞)</strong>, meaning "Stripe" or "Pattern." This library is designed for developers who need a fast, type-safe, and minimal-dependency way to integrate Stripe payments into their Rust applications.
  </p>
</div>

## Is Shima Right for You?
Shima is a lightweight, high-performance Stripe API client library written in Rust. It is designed for developers who need a fast, type-safe, and minimal-dependency way to integrate Stripe payments into their Rust applications. With that being said, Shima is not suitable for all use cases. Here are some scenarios where Shima might be a good fit:

1. You use Stripe for checkouts and customer management.
2. You use Stripe for Subscriptions.
3. You use Stripe for Webhooks regarding Subscriptions.

## Benefits
* Shima compiles up to 10x faster than `async-stripe`.
* Fast
* Type-safe
* Minimal dependencies
* Easy to use
* Macros remove all the boilerplate code

## Getting Started
Add `shima` to your `Cargo.toml` file:

```sh
# gives you the basics including macros
cargo add shima
# if you want to use this with sqlx
cargo add shima --features sqlx
# if you want to use this with webhooks
cargo add shima --features webhooks
# or you can enable all features
cargo add shima --features all
```


## Usage
### Generating a new shima client
```rust
// You can generate a client directly from your environment variables
// if you have `STRIPE_SECRET_KEY` and will also set the webhook if
// `STRIPE_WEBHOOK_SECRET` is set. This is preferred.
let client = shima::Client::from_env();
// Alternatively, you can load it from a string without a webhook secret.
let client = shima::Client::new("sk_test_123456...");
// Or, you can load it from a string with a webhook secret.
let client = shima::Client::new("sk_test_123456...").with_webhook_secret("whsec_123456...");
```

### Creating a Stripe Customer
There are two ways that you can create a Customer in Stripe. You can either use the `create_customer!` macro or go through the `Customer` struct.

```rs
/// This creates a new Customer in Stripe with the macro.
async fn macro_example() -> Result<Customer, shima::Error> {
	let client = Client::from_env();
	create_customer!(&client, "John Doe", "john@example.com")
}

// Create a Customer in Stripe.
async fn code_example() -> Result<Customer, shima::Error> {
    let client = Client::from_env();
    let mut customer = CreateCustomer::new("John Doe", "john@example.com");
    Customer::create(&client, customer).await
}
```

### Purchasing Subscriptions / Checkout
There are two ways that you can create a Checkout Session for a Customer. You can either use the 
`checkout!` macro or go through the `CheckoutSession` struct.

```rust
/// This creates a new Checkout Session for a Customer with the macro.
async fn macro_example() -> Result<CheckoutSession, shima::Error> {
	let client = Client::from_env();
	create_checkout!(&client, "cus_123456", "price_123456", "https://example.com/success", "https://example.com/cancel")
}

// Create a Checkout Session for a Customer.
async fn create_checkout_session() -> Result<CheckoutSession, shima::Error> {
    let client = Client::from_env();

    // Setup the Checkout Session.
    let mut session = CreateCheckoutSession::new_subscription(
        CustomerId::try_from("cus_1234567")?,
        PriceId::try_from("price_1234567")?,
        SuccessUrl::from("https://example.com/success"),
        CancelUrl::from("https://example.com/cancel"),
    );

    // Create the Checkout Session.
    CheckoutSession::create(&client, session).await
}
```

### Manage Subscriptions / Customer Portal
```rust
/// Bring the customer to their Customer Portal via the macro.
async fn macro_example() -> Result<CustomerPortalSession, shima::Error> {
	let client = Client::from_env();
	manage_subscriptions!(&client, "cus_123456", "https://example.com")
}

/// Bring the customer to their Customer Portal via code.
async fn manage_subscription() -> Result<CustomerPortalSession, shima::Error> {
    // Generate a new shima client, reading from our environment variables.
    let client = Client::from_env();

    // Get the customer you want to manage.
    let customer = CustomerId::try_from("cus_123456")?;
    // When the customer is done with their session, they'll be redirected to this URL.
    let return_url = ReturnUrl::from("https://example.com");

    // Create the Customer Portal Session.
    let session = CreateCustomerPortalSession::new(customer, return_url);

    CustomerPortalSession::create(&client, session).await
}
```

### Webhooks
```rust
use shima::webhook::ShimaEvent;

// Listen to Stripe events via webhooks
fn listen_to_webhooks(headers: &http::HeaderMap, body: &str) -> Result<(), shima::Error> {
    // Generate a new shima client, reading from our environment variables.
    let client = shima::Client::from_env();
    let listener = shima::webhook::Listener::new(client);
    
    match listener.process(headers, body)? {
		ShimaEvent::CheckoutSessionCompleted(event) => println!("Checkout session completed: {:?}", event),
		ShimaEvent::InvoicePaymentFailed(event) => println!("Invoice payment failed: {:?}", event),
		ShimaEvent::CustomerSubscriptionDeleted(event) => println!("Customer subscription deleted: {:?}", event),
		ShimaEvent::Other(event) => println!("Other event: {:?}", event),
    }

    Ok(())
}
```
