//! Builds `application/x-www-form-urlencoded` bodies in Stripe's
//! bracket notation (`metadata[key]=value`, `line_items[0][price]=...`).

use crate::Metadata;

#[derive(Debug, Default, Clone)]
pub(crate) struct Form(Vec<(String, String)>);

impl Form {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, key: impl Into<String>, value: impl ToString) -> &mut Self {
        self.0.push((key.into(), value.to_string()));
        self
    }

    pub fn push_opt(&mut self, key: impl Into<String>, value: Option<impl ToString>) -> &mut Self {
        if let Some(value) = value {
            self.push(key, value);
        }
        self
    }

    /// Pushes a string value unless it is empty.
    pub fn push_non_empty(&mut self, key: impl Into<String>, value: &str) -> &mut Self {
        if !value.is_empty() {
            self.push(key, value);
        }
        self
    }

    /// Pushes every metadata entry as `{prefix}[key]=value`.
    pub fn push_metadata(&mut self, prefix: &str, metadata: &Metadata) -> &mut Self {
        for (k, v) in metadata.iter() {
            self.push(format!("{prefix}[{k}]"), v);
        }
        self
    }

    pub fn pairs(&self) -> &[(String, String)] {
        &self.0
    }

    #[cfg(test)]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }
}
