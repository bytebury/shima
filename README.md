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

async create_customer() -> Result<Customer, ripe::Error> {
    let ripe_client = ripe::Client::new_from_env();

    Customer::create(
        CreateCustomer {
            name: "John Doe",
            email: "john.doe@example.com",
            ..Default::default(),
        }
    ).await
}
```
