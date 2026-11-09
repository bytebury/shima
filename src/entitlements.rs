//! Feature gating with Stripe Entitlements: attach features to products in
//! Stripe, then check which ones a customer currently has.
//! See <https://docs.stripe.com/billing/entitlements>.

use crate::{util, CustomerId, IntoId, List};
use serde::{Deserialize, Serialize};

/// A feature a customer currently has access to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveEntitlement {
    pub id: String,
    /// The feature's ID (`feat_...`).
    #[serde(default, deserialize_with = "util::expandable_id")]
    pub feature: Option<String>,
    /// The lookup key you gave the feature, e.g. `sso` or `api_access`.
    pub lookup_key: String,
}

/// The features a customer currently has access to.
///
/// ```no_run
/// use shima::entitlements::Entitlements;
/// # async fn run(client: &shima::Client) -> Result<(), shima::Error> {
/// let entitlements = Entitlements::for_customer(client, "cus_123").await?;
/// if entitlements.has("sso") {
///     // show the SSO settings
/// }
/// # Ok(())
/// # }
/// ```
///
/// Stripe sends `entitlements.active_entitlement_summary.updated` webhooks
/// when these change, so you can cache them and refresh on that event.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Entitlements(pub Vec<ActiveEntitlement>);

impl Entitlements {
    pub async fn for_customer(
        client: &crate::Client,
        customer: impl IntoId<CustomerId>,
    ) -> Result<Entitlements, crate::Error> {
        let customer = customer.into_id()?;
        let mut all = Vec::new();
        let mut starting_after: Option<String> = None;

        loop {
            let mut query = vec![("customer", customer.as_str()), ("limit", "100")];
            if let Some(after) = starting_after.as_deref() {
                query.push(("starting_after", after));
            }
            let page: List<ActiveEntitlement> = client
                .get("/entitlements/active_entitlements", &query)
                .await?;
            starting_after = page.data.last().map(|e| e.id.clone());
            let has_more = page.has_more;
            all.extend(page.data);
            if !has_more || starting_after.is_none() {
                break;
            }
        }

        Ok(Entitlements(all))
    }

    /// `true` if the customer has the feature with this lookup key.
    pub fn has(&self, lookup_key: &str) -> bool {
        self.0.iter().any(|e| e.lookup_key == lookup_key)
    }

    /// The lookup keys of all the customer's features.
    pub fn lookup_keys(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|e| e.lookup_key.as_str())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
