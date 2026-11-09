pub mod ids;
pub mod list;
pub mod metadata;
pub mod urls;

pub use ids::*;
pub use list::*;
pub use metadata::*;
pub use urls::*;

/// Backwards-compatible paths for the ID types.
pub mod customer {
    pub use super::ids::CustomerId;
}

/// Backwards-compatible paths for the ID types.
pub mod price {
    pub use super::ids::PriceId;
}
