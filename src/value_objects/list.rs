use serde::{Deserialize, Serialize};

/// A page of results from a Stripe list endpoint.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct List<T> {
    #[serde(default = "Vec::new")]
    pub data: Vec<T>,
    /// `true` if there are more results than were returned.
    #[serde(default)]
    pub has_more: bool,
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self { data: Vec::new(), has_more: false }
    }
}

impl<T> IntoIterator for List<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}
