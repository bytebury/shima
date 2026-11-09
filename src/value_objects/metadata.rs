use std::collections::HashMap;

/// Key/value pairs attached to a Stripe object. Handy for linking Stripe
/// objects back to your own records, e.g. `user_id`.
///
/// ```
/// use shima::Metadata;
///
/// let user_id = 42;
/// let metadata = Metadata::new().with("user_id", user_id.to_string()).with("plan", "pro");
/// let metadata: Metadata = [("user_id", "42"), ("plan", "pro")].into();
/// # assert_eq!(metadata.len(), 2);
/// ```
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Metadata(Vec<(String, String)>);

impl Metadata {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Adds a key/value pair. Setting an existing key replaces its value.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let (key, value) = (key.into(), value.into());
        match self.0.iter_mut().find(|(k, _)| *k == key) {
            Some(entry) => entry.1 = value,
            None => self.0.push((key, value)),
        }
    }

    /// Builder-style [`Metadata::insert`].
    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.insert(key, value);
        self
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    pub(crate) fn extend(&mut self, other: Metadata) {
        for (k, v) in other.0 {
            self.insert(k, v);
        }
    }
}

impl<K: Into<String>, V: Into<String>> FromIterator<(K, V)> for Metadata {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut metadata = Metadata::new();
        for (k, v) in iter {
            metadata.insert(k, v);
        }
        metadata
    }
}

impl<K: Into<String>, V: Into<String>, const N: usize> From<[(K, V); N]> for Metadata {
    fn from(pairs: [(K, V); N]) -> Self {
        pairs.into_iter().collect()
    }
}

impl<K: Into<String>, V: Into<String>> From<Vec<(K, V)>> for Metadata {
    fn from(pairs: Vec<(K, V)>) -> Self {
        pairs.into_iter().collect()
    }
}

impl From<HashMap<String, String>> for Metadata {
    fn from(map: HashMap<String, String>) -> Self {
        map.into_iter().collect()
    }
}
