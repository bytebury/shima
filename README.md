<div align="center">
  <img src="https://raw.githubusercontent.com/bytebury/asset-manager/refs/heads/main/zebra.svg" alt="shima logo" width="128" />
  <h1>Shima (縞)</h1>
  <p>
    <strong>Shima</strong> is a lightweight, type-safe Stripe client for Rust, built for
    software-as-a-service apps. The name comes from the Japanese word <strong>Shima (縞)</strong>,
    meaning "Stripe" or "Pattern."
  </p>
</div>

## Is Shima right for you?
Shima covers what a typical SaaS needs from Stripe, and nothing else:

1. Customers
2. Checkout for subscriptions and one-time payments (trials, promo codes, seats)
3. Managing subscriptions (upgrade, downgrade, cancel, reactivate, seats)
4. The Stripe customer portal
5. Usage-based billing (meter events)
6. Feature gating with Stripe Entitlements
7. Verified, typed webhooks

If you need the whole Stripe API, use [`async-stripe`](https://crates.io/crates/async-stripe).
Shima compiles up to 10x faster and stays out of your way.

## Getting started

```sh
cargo add shima                     # customers, checkout, subscriptions, portal, usage, entitlements
cargo add shima --features webhook  # + webhook verification
cargo add shima --features sqlx     # + store IDs (CustomerId, PriceId, SubscriptionId) with sqlx 0.9
cargo add shima --features all      # everything
```

Set `STRIPE_SECRET_KEY` (and `STRIPE_WEBHOOK_SECRET` for webhooks), then:

```rust
use shima::prelude::*;

let client = Client::from_env();          // panics if STRIPE_SECRET_KEY is missing
let client = Client::try_from_env()?;     // ...or handle it yourself
let client = Client::new("sk_test_...").with_webhook_secret("whsec_...");
```

`Client` is cheap to clone. Keep one in your app state.

## Macros
The macros cover the common calls in one line. They **await for you** and
return a `Result`, so put a `?` after them. IDs can be `&str`, `String`,
`&String` or typed IDs, and an invalid ID returns an error instead of
panicking.

After the required arguments you can add **any builder option** as
`name = value`:

```rust
use shima::prelude::*;

async fn example(client: &Client, user_id: i64) -> Result<(), shima::Error> {
    // Customers
    let customer = create_customer!(client, "Jane Doe", "jane@example.com",
        metadata = [("user_id", user_id.to_string())],
    )?;

    // Subscription checkout: redirect the user to `session.url`
    let session = checkout!(client, &customer.id, "price_pro_monthly",
        "https://example.com/success?session_id={CHECKOUT_SESSION_ID}",
        "https://example.com/pricing",
        trial_days = 14,
        allow_promotion_codes = true,
        client_reference_id = user_id,
    )?;

    // One-time payment (lifetime deals, credit packs, ...)
    let session = checkout_payment!(client, &customer.id, "price_credits",
        "https://example.com/success", "https://example.com/pricing",
        quantity = 3,
    )?;

    // Customer portal: let customers manage plans, cards and invoices
    let portal = manage_subscriptions!(client, &customer.id, "https://example.com/account")?;

    // Usage-based billing
    report_usage!(client, "api_requests", &customer.id, 1)?;

    delete_customer!(client, &customer.id)?;
    Ok(())
}
```

| Macro | Required arguments | Options from |
|---|---|---|
| `create_customer!` | name, email | `CreateCustomer` |
| `delete_customer!` | customer | |
| `checkout!` | customer, price, success URL, cancel URL | `CreateCheckoutSession` |
| `checkout_payment!` | customer, price, success URL, cancel URL | `CreateCheckoutSession` |
| `manage_subscriptions!` | customer, return URL | `CreateCustomerPortalSession` |
| `report_usage!` | event name, customer, value | `CreateMeterEvent` |

`create_customer!`, `checkout!` and `manage_subscriptions!` also accept a
prebuilt parameter struct: `checkout!(client, session)`.

### Checkout options

| Option | What it does |
|---|---|
| `trial_days = 14` | Free trial |
| `no_card_required = true` | Trial without a card on file |
| `allow_promotion_codes = true` | Let customers enter promo codes |
| `quantity = 5` | Seats |
| `client_reference_id = user.id` | Your ID, returned in the webhook |
| `metadata = [("k", "v")]` | Metadata on the session |
| `subscription_metadata = [("k", "v")]` | Metadata on the created subscription |
| `automatic_tax = false` | Turn off Stripe Tax (on by default) |
| `tax_id_collection = true` | Collect VAT/GST IDs from businesses |
| `customer_email = "a@b.c"` | Let Stripe create the customer during checkout |
| `billing_address_collection = BillingAddressCollection::Required` | Always ask for an address |

## Without macros
Everything the macros do is available as regular functions:

```rust
use shima::prelude::*;

let customer = Customer::create(&client, CreateCustomer::new("Jane", "jane@example.com")).await?;
let customer = Customer::retrieve(&client, "cus_123").await?;
let customer = Customer::update(&client, "cus_123", UpdateCustomer::new().email("new@example.com")).await?;
let existing = Customer::find_by_email(&client, "jane@example.com").await?;

let session = CheckoutSession::create(&client,
    CreateCheckoutSession::new_subscription(
        "cus_123".try_into()?,
        "price_123".try_into()?,
        "https://example.com/success".into(),
        "https://example.com/cancel".into(),
    )
    .trial_days(14),
).await?;

// On your success page
let session = CheckoutSession::retrieve(&client, &session_id).await?;
if session.is_fulfilled() { /* grant access */ }
```

## Subscriptions

```rust
use shima::prelude::*;

// Does this customer have access?
if let Some(sub) = Subscription::active_for_customer(&client, "cus_123").await? {
    println!("{:?} until {:?}", sub.price_id(), sub.current_period_end());
}

// Upgrade or downgrade
Subscription::change_price(&client, "sub_123", "price_team", ProrationBehavior::CreateProrations).await?;
// Change seats
Subscription::set_quantity(&client, "sub_123", 10, ProrationBehavior::AlwaysInvoice).await?;
// Cancel at the end of the paid period, or undo that
Subscription::cancel_at_period_end(&client, "sub_123").await?;
Subscription::reactivate(&client, "sub_123").await?;
// Cancel right now
Subscription::cancel(&client, "sub_123").await?;
```

Helpers on `Subscription`: `is_active()`, `is_trialing()`, `is_canceling()`,
`price()`, `price_id()`, `quantity()`, `current_period_end()`. The period
helpers work on both old and new (2025-03-31+) Stripe API versions.

## Feature gating with Entitlements
Attach [features](https://docs.stripe.com/billing/entitlements) to your
products in Stripe, then check them in your app:

```rust
use shima::entitlements::Entitlements;

let entitlements = Entitlements::for_customer(&client, "cus_123").await?;
if entitlements.has("sso") {
    // ...
}
```

## Usage-based billing

```rust
use shima::billing::{CreateMeterEvent, MeterEvent};

let event = CreateMeterEvent::new("tokens", "cus_123", 1520)?
    .identifier(request_id); // Stripe drops duplicate identifiers, so retries never double-bill
MeterEvent::create(&client, event).await?;
```

## Webhooks
Requires the `webhook` feature. The listener checks the signature, rejects
events older than 5 minutes (replay protection), and gives you typed objects:

```rust
use shima::webhook::{Listener, ShimaEvent};

fn handle(listener: &Listener, headers: &http::HeaderMap, body: &str) -> Result<(), shima::Error> {
    match listener.process(headers, body)? {
        ShimaEvent::CheckoutSessionCompleted(session) => {
            // session.client_reference_id, session.customer, session.subscription
        }
        ShimaEvent::CustomerSubscriptionUpdated(sub)
        | ShimaEvent::CustomerSubscriptionCreated(sub) => {
            // save sub.status, sub.price_id(), sub.current_period_end()
        }
        ShimaEvent::CustomerSubscriptionDeleted(sub) => { /* revoke access */ }
        ShimaEvent::InvoicePaymentFailed(invoice) => {
            // email the customer invoice.hosted_invoice_url
        }
        ShimaEvent::Other(event) => println!("unhandled {}", event.event_type),
        _ => {}
    }
    Ok(())
}
```

With axum, it looks like this:

```rust
async fn stripe_webhook(
    State(listener): State<Listener>,
    headers: HeaderMap,
    body: String, // the raw body; don't parse it as JSON first
) -> StatusCode {
    match listener.process(&headers, &body) {
        Ok(event) => { /* ... */ StatusCode::OK }
        Err(e) if e.is_webhook_error() => StatusCode::BAD_REQUEST,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
```

Other tools:
- `listener.construct_event(headers, body)` returns the raw `Event` with its
  `id` (for deduplicating retries) and `data.previous_attributes`.
- `listener.process_with_signature(signature, body)` takes the
  `Stripe-Signature` header value directly.
- `Listener::from_secret("whsec_...")` creates a listener without an API key.
- `shima::webhook::generate_test_header(secret, payload)` signs a payload so
  you can test your own handlers.

## Errors
Every call returns `Result<T, shima::Error>`:

```rust
match Customer::retrieve(&client, "cus_123").await {
    Ok(customer) => {}
    Err(e) if e.is_not_found() => {}
    Err(e) if e.is_card_error() => { /* e.stripe_error().unwrap().message is safe to show */ }
    Err(e) => return Err(e.into()),
}
```

## Testing
Point the client at [stripe-mock](https://github.com/stripe/stripe-mock) or
your own server:

```rust
let client = Client::new("sk_test_123").with_base_url("http://localhost:12111/v1");
```

You can also pin an API version with `.with_api_version("...")` or bring
your own `reqwest::Client` with `.with_http_client(...)`.
