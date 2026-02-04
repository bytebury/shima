<div align="center">
  <img src="https://raw.githubusercontent.com/bytebury/asset-manager/refs/heads/main/lemon.svg" alt="ripe logo" width="96" />
  <h1>Ripe</h1>
  <p>A lightweight Stripe library designed for SaaS providers. Enabling easy integration with Stripe's API.</p>
</div>

# Getting Started
TODO

# Usage
## Generating a new ripe client
```rust
// You can generate directly from your environment variables if you
// have `STRIPE_SECRET_KEY` set. This is preferred.
let client = ripe::Client::from_env();
// Alternatively, you can load it from a string.
let client = ripe::Client::new("sk_test_123456...");
```

## Creating a Stripe Customer
```rust
use ripe::customer::{CheckoutSession, CreateCheckoutSession};

// Create a customer in Stripe
async create_customer() -> Result<Customer, ripe::Error> {
    // Generate a new ripe client, reading from our environment variables
    let client = ripe::Client::from_env();

    // Create a customer request struct
    let mut customer = CreateCustomer::new("John Doe", "john.doe@example.com");
    customer.metadata.insert("user_id", "1");

    // Attempt to create the customer
    Customer::create(&client, customer).await
}
```

## Purchasing Subscriptions / Checkout
```rust
todo!();
```

## Manage Subscriptions / Billing Portal
```rust
use ripe::billing::{BillingPortalSession, CreateBillingPortalSession};

// Let customers manage their subscriptions
async manage_subscriptions() -> Result<BillingPortalSession, ripe::Error> {
    // Generate a new ripe client, reading from our environment variables
    let client = ripe::Client::from_env();

    // Create the Billing Portal Session
    let session = CreateBillingPortalSession::new("cus_1234567".try_into()?, "");

    BillingPortalSession::create(&client, session).await
}
```

## Webhooks
```rust
todo!();
```
