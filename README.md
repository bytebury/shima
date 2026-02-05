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

## Getting Started
Add `shima` to your `Cargo.toml` file:

```toml
[dependencies]
shima = "0.1.0"
```

## Usage
### Generating a new shima client
```rust
// You can generate a client directly from your environment variables
// if you have `STRIPE_SECRET_KEY` set. This is preferred.
let client = shima::Client::from_env();
// Alternatively, you can load it from a string.
let client = shima::Client::new("sk_test_123456...");
```

### Creating a Stripe Customer
```rust
use shima::customer::{Customer, CreateCustomer};

// Create a Customer in Stripe.
async fn create_customer() -> Result<Customer, shima::Error> {
    // Generate a new shima client, reading from our environment variables
    let client = shima::Client::from_env();

    // Setup the new Customer.
    let mut customer = CreateCustomer::new("John Doe", "john@example.com");
    customer.metadata.insert("user_id", "123456");

    // Create the customer.
    Customer::create(&client, customer).await
}
```

### Purchasing Subscriptions / Checkout
```rust
use shima::checkout::{CheckoutSession, CreateCheckoutSession};
use shima::{CustomerId, PriceId, CancelUrl, SuccessUrl};

// Create a Checkout Session for a Customer.
async fn create_checkout_session() -> Result<CheckoutSession, shima::Error> {
    // Generate a new shima client, reading from our environment variables
    let client = shima::Client::from_env();

    // Setup the Checkout Session.
    let mut session = CreateCheckoutSession::new_subscription(
        CustomerId::try_from("cus_1234567")?,
        PriceId::try_from("price_1234567")?,
        SuccessUrl::from("https://example.com/success"),
        CancelUrl::from("https://example.com/cancel"),
    );
    session.metadata.insert("user_id", "1"); // Optional metadata

    // Create the Checkout Session.
    CheckoutSession::create(&client, session).await
}

```

### Manage Subscriptions / Customer Portal
```rust
use shima::billing::{CustomerPortalSession, CreateCustomerPortalSession};
use shima::{CustomerId, ReturnUrl};

// Let customers manage their subscriptions
async fn manage_subscription() -> Result<CustomerPortalSession, shima::Error> {
    // Generate a new shima client, reading from our environment variables.
    let client = shima::Client::from_env();

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
todo!();
```
