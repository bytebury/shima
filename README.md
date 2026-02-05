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
TODO

## Usage
### Generating a new shima client
```rust
// You can generate directly from your environment variables if you
// have `STRIPE_SECRET_KEY` set. This is preferred.
let client = shima::Client::from_env();
// Alternatively, you can load it from a string.
let client = shima::Client::new("sk_test_123456...");
```

### Creating a Stripe Customer
```rust
use shima::customer::{Customer, CreateCustomer};

// Create a customer in Stripe
async create_customer() -> Result<Customer, shima::Error> {
    // Generate a new shima client, reading from our environment variables
    let client = shima::Client::from_env();

    // Create a customer request struct
    let mut customer = CreateCustomer::new("John Doe", "john.doe@example.com");
    customer.metadata.insert("user_id", "1");

    // Attempt to create the customer
    Customer::create(&client, customer).await
}
```

### Purchasing Subscriptions / Checkout
```rust
use shima::checkout::{CheckoutSession, CreateCheckoutSession};

// Create a checkout session for a customer
async create_checkout_session() -> Result<CheckoutSession, shima::Error> {
    // Generate a new shima client, reading from our environment variables
    let client = shima::Client::from_env();
    
    let success_url = "https://example.com/success";
    let cancel_url = "https://example.com/cancel";
    let customer = CustomerId::try_from("cus_1234567")?;
    let pro_subscription = PriceId::try_from("price_1234567")?;

    let mut session = CreateCheckoutSession::new_subscription(
        success_url,
        cancel_url,
        customer,
        pro_subscription
    );
    session.metadata.insert("user_id", "1");

    CheckoutSession::create(&client, session).await
}
```

### Manage Subscriptions / Billing Portal
```rust
use shima::billing::{BillingPortalSession, CreateBillingPortalSession};

// Let customers manage their subscriptions
async manage_subscriptions() -> Result<BillingPortalSession, shima::Error> {
    // Generate a new shima client, reading from our environment variables
    let client = shima::Client::from_env();
    let custome = CustomerId::try_from("cus_1234567")?;

    // Create the Billing Portal Session
    let session = CreateBillingPortalSession::new(customer, "https://example.com");

    BillingPortalSession::create(&client, session).await
}
```

### Webhooks
```rust
todo!();
```
