<div align="center">
  <img src="https://raw.githubusercontent.com/bytebury/asset-manager/refs/heads/main/lemon.svg" alt="ripe logo" width="96" />
  <h1>Ripe</h1>
  <p>A lightweight Stripe library designed for SaaS providers. Enabling easy integration with Stripe's API.</p>
</div>

# Getting Started
TODO

# Usage
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
