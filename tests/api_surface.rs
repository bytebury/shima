//! Compile-only checks that the README examples keep working.
#![cfg(feature = "webhook")]
#![allow(unused, clippy::all)]
use shima::prelude::*;

async fn macros(client: &Client, user_id: i64) -> Result<(), shima::Error> {
    let customer = create_customer!(client, "Jane Doe", "jane@example.com",
        metadata = [("user_id", user_id.to_string())],
    )?;
    let session = checkout!(client, &customer.id, "price_pro_monthly",
        "https://example.com/success?session_id={CHECKOUT_SESSION_ID}",
        "https://example.com/pricing",
        trial_days = 14, allow_promotion_codes = true, client_reference_id = user_id,
        no_card_required = true, quantity = 5, subscription_metadata = [("k", "v")],
        automatic_tax = false, tax_id_collection = true,
        billing_address_collection = BillingAddressCollection::Required,
    )?;
    let session = checkout!(client, &customer.id, "price_1", "https://s", "https://c", customer_email = "a@b.c")?;
    let session = checkout_payment!(client, &customer.id, "price_credits",
        "https://example.com/success", "https://example.com/pricing", quantity = 3)?;
    let portal = manage_subscriptions!(client, &customer.id, "https://example.com/account")?;
    report_usage!(client, "api_requests", &customer.id, 1)?;
    delete_customer!(client, &customer.id)?;
    Ok(())
}

async fn functions(client: Client, session_id: String, request_id: String) -> Result<(), shima::Error> {
    let customer = Customer::create(&client, CreateCustomer::new("Jane", "jane@example.com")).await?;
    let customer = Customer::retrieve(&client, "cus_123").await?;
    let customer = Customer::update(&client, "cus_123", UpdateCustomer::new().email("new@example.com")).await?;
    let existing = Customer::find_by_email(&client, "jane@example.com").await?;
    let session = CheckoutSession::create(&client,
        CreateCheckoutSession::new_subscription(
            "cus_123".try_into()?, "price_123".try_into()?,
            "https://example.com/success".into(), "https://example.com/cancel".into(),
        ).trial_days(14),
    ).await?;
    let session = CheckoutSession::retrieve(&client, &session_id).await?;
    if let Some(sub) = Subscription::active_for_customer(&client, "cus_123").await? {
        println!("{:?} until {:?}", sub.price_id(), sub.current_period_end());
    }
    Subscription::change_price(&client, "sub_123", "price_team", ProrationBehavior::CreateProrations).await?;
    Subscription::set_quantity(&client, "sub_123", 10, ProrationBehavior::AlwaysInvoice).await?;
    Subscription::cancel_at_period_end(&client, "sub_123").await?;
    Subscription::reactivate(&client, "sub_123").await?;
    Subscription::cancel(&client, "sub_123").await?;
    let e = Entitlements::for_customer(&client, "cus_123").await?;
    let event = CreateMeterEvent::new("tokens", "cus_123", 1520)?.identifier(request_id);
    MeterEvent::create(&client, event).await?;
    let c = Client::try_from_env()?;
    Ok(())
}

fn webhook(listener: &Listener, headers: &http::HeaderMap, body: &str) -> Result<(), shima::Error> {
    match listener.process(headers, body)? {
        ShimaEvent::CheckoutSessionCompleted(session) => {}
        ShimaEvent::CustomerSubscriptionUpdated(sub) | ShimaEvent::CustomerSubscriptionCreated(sub) => {}
        ShimaEvent::CustomerSubscriptionDeleted(sub) => {}
        ShimaEvent::InvoicePaymentFailed(invoice) => {}
        ShimaEvent::Other(event) => println!("unhandled {}", event.event_type),
        _ => {}
    }
    Ok(())
}

async fn errors(client: Client) -> Result<(), Box<dyn std::error::Error>> {
    match Customer::retrieve(&client, "cus_123").await {
        Ok(customer) => {}
        Err(e) if e.is_not_found() => {}
        Err(e) if e.is_card_error() => {}
        Err(e) => return Err(e.into()),
    }
    Ok(())
}
