use serde::{Serialize, Serializer, ser::SerializeMap};

#[derive(Debug, Default)]
pub struct Metadata<'a>(Vec<(&'a str, &'a str)>);

impl<'a> Metadata<'a> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn insert(&mut self, key: &'a str, value: &'a str) {
        self.0.push((key, value));
    }
}

impl<'a> Serialize for Metadata<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (k, v) in &self.0 {
            let key = format!("metadata[{}]", k);
            map.serialize_entry(&key, v)?;
        }
        map.end()
    }
}
