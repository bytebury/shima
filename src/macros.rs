//! Shortcuts for the most common Stripe calls.
//!
//! Every macro **awaits for you** and returns a `Result`, so use them inside
//! an `async` function and follow them with `?`.
//!
//! IDs can be given as `&str`, `String`, `&String` or typed IDs. An invalid
//! ID returns [`Error::InvalidInput`](crate::Error::InvalidInput) instead of
//! panicking.
//!
//! After the required arguments you can add any builder option as
//! `name = value`. Each option calls the builder method of the same name, so
//! every method on the parameter type works:
//!
//! ```no_run
//! # use shima::prelude::*;
//! # async fn run(client: &Client) -> Result<(), Error> {
//! let session = checkout!(
//!     client, "cus_123", "price_123",
//!     "https://example.com/success", "https://example.com/pricing",
//!     trial_days = 14,
//!     allow_promotion_codes = true,
//!     quantity = 5,
//! )?;
//! # Ok(())
//! # }
//! ```

/// Creates a customer.
///
/// ```no_run
/// # use shima::prelude::*;
/// # async fn run(client: &Client, user_id: i64) -> Result<(), Error> {
/// let customer = create_customer!(client, "Jane Doe", "jane@example.com")?;
///
/// // With options (any `CreateCustomer` builder method):
/// let customer = create_customer!(
///     client, "Jane Doe", "jane@example.com",
///     phone = "+15555550123",
///     metadata = [("user_id", user_id.to_string())],
/// )?;
///
/// // Or with a prebuilt `CreateCustomer`:
/// let customer = create_customer!(client, CreateCustomer::new("Jane", "jane@example.com"))?;
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! create_customer {
    ($client:expr, $name:expr, $email:expr $(, $opt:ident = $val:expr)* $(,)?) => {
        $crate::customer::Customer::create(
            $client,
            $crate::customer::CreateCustomer::new($name, $email) $(.$opt($val))*,
        )
        .await
    };

    ($client:expr, $customer:expr $(,)?) => {
        $crate::customer::Customer::create($client, $customer).await
    };
}

/// Permanently deletes a customer.
///
/// ```no_run
/// # use shima::prelude::*;
/// # async fn run(client: &Client) -> Result<(), Error> {
/// delete_customer!(client, "cus_123")?;
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! delete_customer {
    ($client:expr, $customer_id:expr $(,)?) => {
        $crate::customer::Customer::delete($client, $customer_id).await
    };
}

/// Creates a subscription checkout session. Redirect the customer to
/// `session.url`.
///
/// Arguments: client, customer ID, price ID, success URL, cancel URL, then
/// any [`CreateCheckoutSession`](crate::checkout::CreateCheckoutSession)
/// option as `name = value`.
///
/// ```no_run
/// # use shima::prelude::*;
/// # async fn run(client: &Client) -> Result<(), Error> {
/// let session = checkout!(
///     client, "cus_123", "price_123",
///     "https://example.com/success?session_id={CHECKOUT_SESSION_ID}",
///     "https://example.com/pricing",
/// )?;
///
/// let session = checkout!(
///     client, "cus_123", "price_123",
///     "https://example.com/success", "https://example.com/pricing",
///     trial_days = 14,
///     no_card_required = true,
///     client_reference_id = "team_42",
/// )?;
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! checkout {
    ($client:expr, $customer:expr, $price:expr, $success:expr, $cancel:expr $(, $opt:ident = $val:expr)* $(,)?) => {
        async {
            let session = $crate::checkout::CreateCheckoutSession::new_subscription(
                $crate::IntoId::<$crate::CustomerId>::into_id($customer)?,
                $crate::IntoId::<$crate::PriceId>::into_id($price)?,
                ::core::convert::Into::into($success),
                ::core::convert::Into::into($cancel),
            ) $(.$opt($val))*;
            $crate::checkout::CheckoutSession::create($client, session).await
        }
        .await
    };

    ($client:expr, $session:expr $(,)?) => {
        $crate::checkout::CheckoutSession::create($client, $session).await
    };
}

/// Creates a one-time payment checkout session, e.g. for a lifetime plan or
/// credits. Takes the same arguments as [`checkout!`].
///
/// ```no_run
/// # use shima::prelude::*;
/// # async fn run(client: &Client) -> Result<(), Error> {
/// let session = checkout_payment!(
///     client, "cus_123", "price_credits_100",
///     "https://example.com/success", "https://example.com/pricing",
///     quantity = 2,
/// )?;
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! checkout_payment {
    ($client:expr, $customer:expr, $price:expr, $success:expr, $cancel:expr $(, $opt:ident = $val:expr)* $(,)?) => {
        async {
            let session = $crate::checkout::CreateCheckoutSession::new_payment(
                $crate::IntoId::<$crate::CustomerId>::into_id($customer)?,
                $crate::IntoId::<$crate::PriceId>::into_id($price)?,
                ::core::convert::Into::into($success),
                ::core::convert::Into::into($cancel),
            ) $(.$opt($val))*;
            $crate::checkout::CheckoutSession::create($client, session).await
        }
        .await
    };
}

/// Creates a customer portal session, where customers manage their
/// subscription, payment methods and invoices. Redirect them to `session.url`.
///
/// ```no_run
/// # use shima::prelude::*;
/// # async fn run(client: &Client) -> Result<(), Error> {
/// let session = manage_subscriptions!(client, "cus_123", "https://example.com/account")?;
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! manage_subscriptions {
    ($client:expr, $customer_id:expr, $return_url:expr $(, $opt:ident = $val:expr)* $(,)?) => {
        async {
            let session = $crate::billing::CreateCustomerPortalSession::new(
                $crate::IntoId::<$crate::CustomerId>::into_id($customer_id)?,
                ::core::convert::Into::into($return_url),
            ) $(.$opt($val))*;
            $crate::billing::CustomerPortalSession::create($client, session).await
        }
        .await
    };

    ($client:expr, $session:expr $(,)?) => {
        $crate::billing::CustomerPortalSession::create($client, $session).await
    };
}

/// Reports usage for usage-based billing.
///
/// Arguments: client, meter event name, customer ID, value, then any
/// [`CreateMeterEvent`](crate::billing::CreateMeterEvent) option as
/// `name = value`.
///
/// ```no_run
/// # use shima::prelude::*;
/// # async fn run(client: &Client) -> Result<(), Error> {
/// report_usage!(client, "api_requests", "cus_123", 1)?;
/// report_usage!(client, "tokens", "cus_123", 1520, identifier = "req_8f3a")?;
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! report_usage {
    ($client:expr, $event_name:expr, $customer:expr, $value:expr $(, $opt:ident = $val:expr)* $(,)?) => {
        async {
            let event = $crate::billing::CreateMeterEvent::new($event_name, $customer, $value)?
                $(.$opt($val))*;
            $crate::billing::MeterEvent::create($client, event).await
        }
        .await
    };
}
