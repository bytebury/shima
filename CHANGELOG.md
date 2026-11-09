# Changelog

## 0.7.0

### Security
- **Webhooks without a `Stripe-Signature` header were accepted as valid.**
  Anyone who knew your webhook URL could forge events such as
  `checkout.session.completed`. They are now rejected. Upgrade right away.
- Webhook timestamps older than 5 minutes are rejected (replay protection),
  as in Stripe's official libraries. Configure with `Listener::with_tolerance`.
- Every `v1` signature in the header is checked, so rolling your webhook
  secret no longer causes failures.
- IDs are checked for invalid characters before they go into request paths.
- `Client`'s `Debug` output redacts secrets.

### Macros
- Macros return `Error::InvalidInput` for invalid IDs instead of panicking.
- `manage_subscriptions!(&client, "cus_...", url)` compiles now (it didn't
  accept string IDs).
- Every macro accepts trailing builder options:
  `checkout!(&client, customer, price, ok, cancel, trial_days = 14)`.
- IDs and strings can be passed as `&str`, `String`, `&String`, IDs, or `&ID`.
- New: `checkout_payment!`, `report_usage!`.
- `use shima::prelude::*;` brings in all macros and common types.

### New features
- **Subscriptions**: `retrieve`, `list_for_customer`, `active_for_customer`,
  `cancel`, `cancel_at_period_end`, `reactivate`, `change_price`,
  `set_quantity`, plus `is_active()`, `current_period_end()` and other helpers.
- **Customers**: `retrieve`, `update`, `find_by_email`; `Address` type.
- **Checkout**: trials, card-less trials, promotion codes, quantities/seats,
  extra line items, session and subscription metadata, tax ID collection,
  turning off automatic tax, `customer_email`, one-time payments
  (`new_payment`), `CheckoutSession::retrieve`, `is_fulfilled()`; sessions now
  include `customer`, `subscription`, `status`, `payment_status` and more.
- **Usage-based billing**: `billing::MeterEvent`.
- **Entitlements**: `entitlements::Entitlements::for_customer(..).has("feature")`.
- **Webhooks**: typed events (`Subscription`, `Invoice`, `CheckoutSession`,
  `Customer`), more event types, `construct_event` for the raw `Event` and
  its ID, `process_with_signature`, `Listener::from_secret`,
  `generate_test_header` for testing your handlers, and byte payloads.
- **Client**: `try_from_env`, `with_api_version`, `with_base_url`
  (stripe-mock), `with_http_client`, `is_test_mode`.
- **Errors**: `InvalidInput`, `Config` and `Webhook` variants; HTTP status,
  `decline_code` and `doc_url` on Stripe errors; `is_not_found()`,
  `is_card_error()`, `is_webhook_error()`.
- `SubscriptionId`; IDs implement `PartialEq`, `Eq`, `Hash`, `FromStr`.
- `webhooks` is accepted as an alias for the `webhook` feature (the README
  used to say `webhooks`).

### Fixes
- `Customer` failed to deserialize when the customer had an address.
- `customer_update[address]` was sent without a `customer`, which Stripe
  rejects.
- `Mode`, `LineItem` and the other checkout types weren't importable.

### Breaking changes
- `ShimaEvent` variants hold typed objects instead of `serde_json::Value`,
  and the enum is `#[non_exhaustive]` (add a `_ =>` arm). `Other` holds an
  `Event`; its object is in `event.data.object`.
- `Error` is `#[non_exhaustive]`, and `Error::Stripe` holds a
  `Box<StripeError>` (field access like `e.message` still works).
- Webhook failures return `Error::Webhook`; a missing secret returns `Error::Config`.
- `Metadata` owns its strings and has no lifetime parameter.
- `CreateCustomer` fields are `Cow<str>`; `CreateCustomerPortalSession` has
  a `configuration` field; `CustomerUpdate` and `AutomaticTax` changed shape.
- `CheckoutSession` and `Customer` have new fields (affects struct literals).
