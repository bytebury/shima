#[macro_export]
macro_rules! create_customer {
    ($client:expr, $name:expr, $email:expr) => {{
        let data = $crate::customer::CreateCustomer {
            name: $name.into(),
            email: $email.into(),
            ..Default::default()
        };
        $crate::customer::Customer::create($client, data).await
    }};

    ($client:expr, $customer:expr) => {
        $crate::customer::Customer::create($client, $customer).await
    };
}

#[macro_export]
macro_rules! checkout {
    ($client:expr, $customer:expr, $price:expr, $success:expr, $cancel:expr) => {{
        let session = $crate::checkout::CreateCheckoutSession::new_subscription(
            $customer.try_into().unwrap(),
            $price.try_into().unwrap(),
            $success.into(),
            $cancel.into(),
        );
        $crate::checkout::CheckoutSession::create($client, session).await
    }};

    ($client:expr, $session:expr) => {
        $crate::CheckoutSession::create($client, $session).await
    };
}

#[macro_export]
macro_rules! manage_subscriptions {
    ($client:expr, $customer_id:expr, $return_url:expr) => {{
        let session = $crate::CreateCustomerPortalSession {
            customer: $customer_id,
            return_url: Some($return_url),
            ..Default::default()
        };
        $crate::CustomerPortalSession::create($client, session).await
    }};

    ($client:expr, $session:expr) => {
        $crate::CustomerPortalSession::create($client, $session).await
    };
}
