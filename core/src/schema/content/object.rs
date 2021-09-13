use super::prelude::*;
use serde::{
    de::{Deserialize, Deserializer},
    ser::Serializer,
};
use std::borrow::Cow;
use std::collections::BTreeMap;

const RESERVED_FIELDS: [&str; 2] = ["type", "skip_when_null"];

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct ObjectContent {
    #[serde(default)]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub skip_when_null: bool,
    #[serde(flatten)]
    #[serde(serialize_with = "normalize_keys")]
    #[serde(deserialize_with = "denormalize_keys")]
    pub fields: BTreeMap<String, Content>,
}

fn normalize_keys<S: Serializer>(
    fields: &BTreeMap<String, Content>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.collect_map(fields.iter().map(|(k, v)| (add_reserved_underscores(k), v)))
}

fn denormalize_keys<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, Content>, D::Error> {
    Ok(BTreeMap::<String, Content>::deserialize(deserializer)?
        .into_iter()
        .map(|(k, v)| (remove_reserved_underscores(k), v))
        .collect())
}

fn add_reserved_underscores(key: &str) -> Cow<str> {
    for reserved_field in RESERVED_FIELDS {
        if key.starts_with(reserved_field) && key[reserved_field.len()..].bytes().all(|b| b == b'_')
        {
            return Cow::Owned(key.to_string() + "_");
        }
    }
    Cow::Borrowed(key)
}

fn remove_reserved_underscores(mut key: String) -> String {
    for reserved_field in RESERVED_FIELDS {
        if key.starts_with(&format!("{}_", reserved_field))
            && key[reserved_field.len() + 1..].bytes().all(|b| b == b'_')
        {
            key.truncate(key.len() - 1);
        }
    }
    key
}

impl ObjectContent {
    pub fn get(&self, field: &str) -> Result<&Content> {
        let suggest = suggest_closest(self.fields.keys(), field).unwrap_or_default();
        self.fields.get(field).ok_or_else(|| {
            failed!(target: Release,
                NotFound => "no such field: '{}'{}",
                field,
                suggest
            )
        })
    }

    pub fn accepts(&self, obj: &JsonObject) -> Result<()> {
        // There is probably a more efficient way of doing this
        // But it's linear time

        // First check if JSON has all the required fields
        for (k, v) in self.iter() {
            if let Some(content) = v.as_nullable() {
                if let Some(value) = obj.get(k) {
                    content.accepts(value)?;
                }
            } else {
                let json_value = obj
                    .get(k)
                    .ok_or_else(|| failed!(target: Release, "could not find field: '{}'", k))?;
                v.accepts(json_value)
                    .with_context(|| anyhow!("in a field: '{}'", k))?;
            }
        }

        // Then check if fields contains all the json keys
        for (k, _) in obj {
            if !self.fields.contains_key(k) {
                return Err(failed!(
                    target: Release,
                    "field '{}' is not recognized in the schema",
                    k
                ));
            }
        }

        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Content)> {
        self.fields.iter()
    }

    pub fn get_mut(&mut self, field: &str) -> Result<&mut Content> {
        let suggest = suggest_closest(self.fields.keys(), field).unwrap_or_default();
        self.fields.get_mut(field).ok_or_else(
            || failed!(target: Release, NotFound => "no such field: '{}'{}", field, suggest),
        )
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Find<Content> for ObjectContent {
    fn project<I, R>(&self, mut reference: Peekable<I>) -> Result<&Content>
    where
        I: Iterator<Item = R>,
        R: AsRef<str>,
    {
        let next_ = reference
            .next()
            .ok_or_else(|| failed!(target: Release, "expected a field name, found nothing"))?;
        let next = next_.as_ref();
        self.get(next)?
            .project(reference)
            .with_context(|| anyhow!("in a field: {}", next))
    }

    fn project_mut<I, R>(&mut self, mut reference: Peekable<I>) -> Result<&mut Content>
    where
        I: Iterator<Item = R>,
        R: AsRef<str>,
    {
        let next_ = reference
            .next()
            .ok_or_else(|| failed!(target: Release, "expected a field name, found nothing"))?;
        let next = next_.as_ref();
        self.get_mut(next)?
            .project_mut(reference)
            .with_context(|| anyhow!("in a field named {}", next))
    }
}

impl Compile for ObjectContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, mut compiler: C) -> Result<Graph> {
        let object_node = self
            .iter()
            .map(|(name, field)| {
                if self.skip_when_null && field.is_nullable() {
                    let nullable = field.as_nullable().unwrap();
                    compiler
                        .build(name, nullable)
                        .map(|graph| KeyValueOrNothing::sometimes(name, graph))
                } else {
                    compiler
                        .build(name, field)
                        .map(|graph| KeyValueOrNothing::always(name, graph))
                }
            })
            .collect::<Result<ObjectNode>>()?;
        Ok(Graph::Object(object_node))
    }
}
