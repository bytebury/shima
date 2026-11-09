//! Verifying and parsing Stripe webhooks.
//! See <https://docs.stripe.com/webhooks>.
//!
//! ```no_run
//! use shima::webhook::{Listener, ShimaEvent};
//!
//! fn handle(listener: &Listener, headers: &http::HeaderMap, body: &str) -> Result<(), shima::Error> {
//!     match listener.process(headers, body)? {
//!         ShimaEvent::CheckoutSessionCompleted(session) => { /* grant access */ }
//!         ShimaEvent::CustomerSubscriptionUpdated(sub) => { /* sync sub.status */ }
//!         ShimaEvent::CustomerSubscriptionDeleted(sub) => { /* revoke access */ }
//!         _ => {}
//!     }
//!     Ok(())
//! }
//! ```

use crate::Error;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use subtle::ConstantTimeEq;

pub mod event;
pub use event::*;

type HmacSha256 = Hmac<Sha256>;

/// How old a webhook's timestamp may be before it is rejected, as in
/// Stripe's official libraries. Protects against replayed requests.
pub const DEFAULT_TOLERANCE: Duration = Duration::from_secs(300);

/// Verifies webhook signatures and parses events.
#[derive(Clone, Debug)]
pub struct Listener {
    client: crate::Client,
    tolerance: Option<Duration>,
}

impl Listener {
    /// Creates a listener using the client's webhook secret.
    pub fn new(client: crate::Client) -> Self {
        Self {
            client,
            tolerance: Some(DEFAULT_TOLERANCE),
        }
    }

    /// Creates a listener from a webhook secret (`whsec_...`) only.
    pub fn from_secret(webhook_secret: impl Into<String>) -> Self {
        Self::new(crate::Client::new(String::new()).with_webhook_secret(webhook_secret))
    }

    /// Changes how old an event's signature timestamp may be.
    pub fn with_tolerance(mut self, tolerance: Duration) -> Self {
        self.tolerance = Some(tolerance);
        self
    }

    /// Turns off the timestamp check. Only do this in tests.
    pub fn without_timestamp_check(mut self) -> Self {
        self.tolerance = None;
        self
    }

    /// Verifies the request and returns a typed [`ShimaEvent`].
    ///
    /// `payload` must be the raw request body, exactly as received. Return
    /// HTTP 400 when this fails with [`Error::Webhook`].
    pub fn process(
        &self,
        headers: &http::HeaderMap,
        payload: impl AsRef<[u8]>,
    ) -> Result<ShimaEvent, Error> {
        self.construct_event(headers, payload)?.into_shima_event()
    }

    /// Like [`Listener::process`], taking the `Stripe-Signature` header value.
    pub fn process_with_signature(
        &self,
        signature: &str,
        payload: impl AsRef<[u8]>,
    ) -> Result<ShimaEvent, Error> {
        self.construct_event_from_signature(signature, payload)?
            .into_shima_event()
    }

    /// Verifies the request and returns the raw [`Event`], including its ID.
    pub fn construct_event(
        &self,
        headers: &http::HeaderMap,
        payload: impl AsRef<[u8]>,
    ) -> Result<Event, Error> {
        let signature = headers
            .get("Stripe-Signature")
            .ok_or_else(|| Error::Webhook("missing Stripe-Signature header".into()))?
            .to_str()
            .map_err(|_| Error::Webhook("Stripe-Signature header is not valid text".into()))?;
        self.construct_event_from_signature(signature, payload)
    }

    /// Like [`Listener::construct_event`], taking the `Stripe-Signature`
    /// header value.
    pub fn construct_event_from_signature(
        &self,
        signature: &str,
        payload: impl AsRef<[u8]>,
    ) -> Result<Event, Error> {
        let payload = payload.as_ref();
        self.verify_signature(signature, payload)?;
        serde_json::from_slice(payload)
            .map_err(|e| Error::Webhook(format!("invalid event payload: {e}")))
    }

    fn verify_signature(&self, header: &str, payload: &[u8]) -> Result<(), Error> {
        let secret = self
            .client
            .stripe_webhook_secret
            .as_deref()
            .ok_or_else(|| Error::Config("STRIPE_WEBHOOK_SECRET is not set".into()))?;

        let (timestamp, signatures) = parse_signature_header(header)
            .ok_or_else(|| Error::Webhook("malformed Stripe-Signature header".into()))?;

        if let Some(tolerance) = self.tolerance {
            let age = now() - timestamp;
            if age > tolerance.as_secs() as i64 {
                return Err(Error::Webhook(format!(
                    "timestamp is {age}s old, outside the {}s tolerance",
                    tolerance.as_secs()
                )));
            }
        }

        let expected = compute_signature(secret, timestamp, payload);

        // Stripe may send several v1 signatures while you roll your secret.
        let valid = signatures
            .iter()
            .filter_map(|s| hex::decode(s).ok())
            .any(|s| s.len() == expected.len() && bool::from(s.ct_eq(&expected)));

        if valid {
            Ok(())
        } else {
            Err(Error::Webhook("signature verification failed".into()))
        }
    }
}

fn parse_signature_header(header: &str) -> Option<(i64, Vec<&str>)> {
    let mut timestamp = None;
    let mut signatures = Vec::new();

    for part in header.split(',') {
        match part.trim().split_once('=') {
            Some(("t", v)) => timestamp = v.parse::<i64>().ok(),
            Some(("v1", v)) => signatures.push(v),
            _ => {}
        }
    }

    match (timestamp, signatures.is_empty()) {
        (Some(t), false) => Some((t, signatures)),
        _ => None,
    }
}

fn compute_signature(secret: &str, timestamp: i64, payload: &[u8]) -> Vec<u8> {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts keys of any size");
    mac.update(timestamp.to_string().as_bytes());
    mac.update(b".");
    mac.update(payload);
    mac.finalize().into_bytes().to_vec()
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Builds a valid `Stripe-Signature` header value for `payload`, signed now.
/// Use it to test your own webhook handlers.
///
/// ```
/// use shima::webhook::{generate_test_header, Listener, ShimaEvent};
///
/// let payload = r#"{"id":"evt_1","type":"ping","data":{"object":{}}}"#;
/// let header = generate_test_header("whsec_test", payload);
///
/// let listener = Listener::from_secret("whsec_test");
/// let event = listener.process_with_signature(&header, payload).unwrap();
/// assert!(matches!(event, ShimaEvent::Other(_)));
/// ```
pub fn generate_test_header(secret: &str, payload: impl AsRef<[u8]>) -> String {
    generate_test_header_at(secret, now(), payload)
}

/// Like [`generate_test_header`] with an explicit Unix timestamp.
pub fn generate_test_header_at(secret: &str, timestamp: i64, payload: impl AsRef<[u8]>) -> String {
    let signature = compute_signature(secret, timestamp, payload.as_ref());
    format!("t={timestamp},v1={}", hex::encode(signature))
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderMap;

    const SECRET: &str = "whsec_test_secret";
    const PAYLOAD: &str = r#"{"id":"evt_1","type":"customer.subscription.deleted","created":1,"livemode":false,"data":{"object":{"id":"sub_1","customer":"cus_1","status":"canceled"}}}"#;

    fn listener() -> Listener {
        Listener::from_secret(SECRET)
    }

    fn headers(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("Stripe-Signature", value.parse().unwrap());
        headers
    }

    #[test]
    fn accepts_valid_signature_and_parses_typed_event() {
        let header = generate_test_header(SECRET, PAYLOAD);
        match listener().process(&headers(&header), PAYLOAD).unwrap() {
            ShimaEvent::CustomerSubscriptionDeleted(sub) => {
                assert_eq!(sub.id, "sub_1");
                assert_eq!(sub.customer, "cus_1");
            }
            other => panic!("unexpected event {other:?}"),
        }
    }

    #[test]
    fn rejects_missing_signature_header() {
        let err = listener().process(&HeaderMap::new(), PAYLOAD).unwrap_err();
        assert!(err.is_webhook_error(), "{err}");
    }

    #[test]
    fn rejects_wrong_secret_and_tampered_payload() {
        let header = generate_test_header("whsec_other", PAYLOAD);
        assert!(listener().process(&headers(&header), PAYLOAD).is_err());

        let header = generate_test_header(SECRET, PAYLOAD);
        let tampered = PAYLOAD.replace("sub_1", "sub_2");
        assert!(listener().process(&headers(&header), tampered).is_err());
    }

    #[test]
    fn rejects_malformed_headers() {
        for header in ["", "t=123", "v1=abcd", "t=abc,v1=abcd", "garbage"] {
            assert!(listener().process(&headers(header), PAYLOAD).is_err(), "{header}");
        }
    }

    #[test]
    fn enforces_timestamp_tolerance() {
        let old = generate_test_header_at(SECRET, now() - 600, PAYLOAD);
        assert!(listener().process(&headers(&old), PAYLOAD).is_err());
        assert!(listener()
            .without_timestamp_check()
            .process(&headers(&old), PAYLOAD)
            .is_ok());
        assert!(listener()
            .with_tolerance(Duration::from_secs(3600))
            .process(&headers(&old), PAYLOAD)
            .is_ok());
    }

    #[test]
    fn accepts_any_matching_v1_signature() {
        let t = now();
        let good = generate_test_header_at(SECRET, t, PAYLOAD);
        let good_sig = good.split("v1=").nth(1).unwrap();
        let header = format!("t={t},v1={},v1={good_sig},v0=ignored", "00".repeat(32));
        assert!(listener().process(&headers(&header), PAYLOAD).is_ok());
    }

    #[test]
    fn requires_webhook_secret() {
        let listener = Listener::new(crate::Client::new("sk_test_1"));
        let header = generate_test_header(SECRET, PAYLOAD);
        assert!(matches!(
            listener.process(&headers(&header), PAYLOAD),
            Err(Error::Config(_))
        ));
    }

    #[test]
    fn unknown_events_are_other_with_raw_event() {
        let payload = r#"{"id":"evt_9","type":"product.created","data":{"object":{"id":"prod_1"}}}"#;
        let header = generate_test_header(SECRET, payload);
        match listener().process_with_signature(&header, payload).unwrap() {
            ShimaEvent::Other(event) => {
                assert_eq!(event.id, "evt_9");
                assert_eq!(event.event_type, "product.created");
            }
            other => panic!("unexpected event {other:?}"),
        }
    }

    #[test]
    fn accepts_byte_payloads() {
        let header = generate_test_header(SECRET, PAYLOAD);
        let bytes: Vec<u8> = PAYLOAD.as_bytes().to_vec();
        assert!(listener().construct_event_from_signature(&header, &bytes).is_ok());
    }
}
