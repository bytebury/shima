use crate::{
    Error,
    webhook::event::{ShimaEvent, StripeEventRequest},
};
use hmac::{Hmac, digest::*};
use sha2::Sha256;
use subtle::ConstantTimeEq;

pub mod event;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct Listener {
    client: crate::Client,
}

impl Listener {
    pub fn new(client: crate::Client) -> Self {
        Self { client }
    }

    pub fn process(
        &self,
        headers: &http::HeaderMap,
        payload: &str,
    ) -> Result<ShimaEvent, crate::Error> {
        if self.client.stripe_webhook_secret.is_none() {
            return Err(Error::Internal(
                "You do not have your STRIPE_WEBHOOK_SECRET set.".to_string(),
            ));
        }

        if !self.verify(headers, payload).is_none_or(|x| x) {
            return Err(Error::Internal("signature verification failed".to_string()));
        }

        let event: StripeEventRequest = serde_json::from_str(payload)?;

        match event.r#type.as_str() {
            "checkout.session.completed" => {
                Ok(ShimaEvent::CheckoutSessionCompleted(event.data.object))
            }
            "customer.subscription.deleted" => {
                Ok(ShimaEvent::CustomerSubscriptionDeleted(event.data.object))
            }
            "invoice.payment_failed" => Ok(ShimaEvent::InvoicePaymentFailed(event.data.object)),
            _ => Ok(ShimaEvent::Other(event.data.object)),
        }
    }

    fn verify(&self, headers: &http::HeaderMap, payload: &str) -> Option<bool> {
        let signature_header = headers.get("Stripe-Signature")?.to_str().ok()?;
        let valid = self.verify_signature(signature_header, payload);

        Some(valid)
    }

    fn verify_signature(&self, signature_header: &str, payload: &str) -> bool {
        let (timestamp, signature_hex) = match self.parse_signature(signature_header) {
            Some(x) => x,
            _ => return false,
        };
        let signed_payload = format!("{timestamp}.{payload}");

        // HMAC
        let mut mac = match <HmacSha256 as KeyInit>::new_from_slice(
            self.client
                .stripe_webhook_secret
                .as_ref()
                .expect("STRIPE_WEBHOOK_SECRET is required")
                .as_bytes(),
        ) {
            Ok(m) => m,
            Err(_) => return false,
        };

        Update::update(&mut mac, signed_payload.as_bytes());
        let expected = mac.finalize().into_bytes();

        // decode header-provided hex signature to bytes
        let sig_bytes = match hex::decode(signature_hex) {
            Ok(v) => v,
            Err(_) => return false,
        };

        if expected.len() != sig_bytes.len() {
            return false;
        }

        // constant-time compare
        expected.as_slice().ct_eq(&sig_bytes).unwrap_u8() == 1
    }

    fn parse_signature(&self, header: &str) -> Option<(String, String)> {
        let mut ts = None;
        let mut sig = None;

        for part in header.split(',') {
            let mut kv = part.splitn(2, '=');
            match (kv.next(), kv.next()) {
                (Some("t"), Some(v)) => ts = Some(v.to_string()),
                // pick the first v1 we see, per API docs
                (Some("v1"), Some(v)) if sig.is_none() => sig = Some(v.to_string()),
                _ => {}
            }
        }
        match (ts, sig) {
            (Some(t), Some(s)) => Some((t, s)),
            _ => None,
        }
    }
}
