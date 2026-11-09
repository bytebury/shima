use serde::{Deserialize, Deserializer};

/// Treats an explicit JSON `null` as the type's default value.
pub(crate) fn null_default<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(d)?.unwrap_or_default())
}

/// Stripe fields like `customer` are either an ID string or, when expanded,
/// the full object. This accepts both and keeps the ID.
pub(crate) fn expandable_id<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: RawId,
{
    Ok(Option::<Expandable>::deserialize(d)?.map(|e| T::from_raw(e.into_id())))
}

/// Same as [`expandable_id`] for fields that are always present.
pub(crate) fn expandable_id_required<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: RawId,
{
    Ok(T::from_raw(Expandable::deserialize(d)?.into_id()))
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Expandable {
    Id(String),
    Object { id: String },
}

impl Expandable {
    fn into_id(self) -> String {
        match self {
            Expandable::Id(id) | Expandable::Object { id } => id,
        }
    }
}

/// Builds an ID from a value Stripe sent us, without validating it.
pub(crate) trait RawId {
    fn from_raw(id: String) -> Self;
}

impl RawId for String {
    fn from_raw(id: String) -> Self {
        id
    }
}
