use super::prelude::*;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt;
use std::ops::Not;
use serde::{ser::{Serialize, Serializer, SerializeMap}, de::{Deserialize, Deserializer, MapAccess}};

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectContent {
    pub fields: BTreeMap<String, FieldContent>,
}

fn add_type_underscore(key: &str) -> Cow<str> {
    if key.starts_with("type") && key[4..].bytes().all(|b| b == b'_') {
        Cow::Owned(key.to_string() + "_")
    } else {
        Cow::Borrowed(key)
    }
}

fn remove_type_underscore(mut key: String) -> String {
    if key.starts_with("type_") && key[5..].bytes().all(|b| b == b'_') {
        key.truncate(key.len() - 1);
    }
    key
}

impl Serialize for ObjectContent {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.fields.len()))?;
        for (k, v) in &self.fields {
            map.serialize_entry(&add_type_underscore(k), v)?;
        }
        map.end()
    }
}

struct ObjectContentVisitor;

impl<'de> Visitor<'de> for ObjectContentVisitor {
    type Value = ObjectContent;
    
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "an object's contents")
    }

    fn visit_map<M: MapAccess<'de>>(self, mut access: M) -> Result<Self::Value, M::Error> {
        let mut fields = BTreeMap::new();
        while let Some((key, value)) = access.next_entry()? {
            let key: String = key;
            if fields.contains_key(&key) {
                return Err(serde::de::Error::custom(format!("duplicate field: {}", &key)));
            }
            fields.insert(remove_type_underscore(key), value);
        }
        Ok(ObjectContent { fields })
    }
}

impl<'de> Deserialize<'de> for ObjectContent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(ObjectContentVisitor)
    }
}

impl ObjectContent {
    pub fn get(&self, field: &str) -> Result<&FieldContent> {
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
            if v.optional {
                if let Some(value) = obj.get(k) {
                    v.content.accepts(value)?;
                }
            } else {
                let json_value =
                    obj.get(k)
                        .ok_or(failed!(target: Release, "could not find field: '{}'", k))?;
                v.content
                    .accepts(json_value)
                    .context(anyhow!("in a field: '{}'", k))?;
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

    pub fn iter(&self) -> impl Iterator<Item = (&String, &FieldContent)> {
        self.fields.iter()
    }

    pub fn get_mut(&mut self, field: &str) -> Result<&mut FieldContent> {
        let suggest = suggest_closest(self.fields.keys(), field).unwrap_or_default();
        self.fields.get_mut(field).ok_or_else(
            || failed!(target: Release, NotFound => "no such field: '{}'{}", field, suggest),
        )
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FieldContent {
    #[serde(default, skip_serializing_if = "Not::not")]
    pub optional: bool,
    #[serde(flatten)]
    pub content: Box<Content>,
}

impl FieldContent {
    pub fn new<I: Into<Content>>(content: I) -> Self {
        FieldContent {
            optional: false,
            content: Box::new(content.into()),
        }
    }

    pub fn optional(&mut self, optional: bool) {
        self.optional = optional;
    }
}

impl Default for ObjectContent {
    fn default() -> Self {
        Self {
            fields: BTreeMap::new(),
        }
    }
}

impl Find<Content> for ObjectContent {
    fn project<I, R>(&self, mut reference: Peekable<I>) -> Result<&Content>
    where
        I: Iterator<Item = R>,
        R: AsRef<str>,
    {
        let next_ = reference.next().ok_or(failed!(
            target: Release,
            "expected a field name, found nothing"
        ))?;
        let next = next_.as_ref();
        self.get(next)?
            .content
            .project(reference)
            .context(anyhow!("in a field: {}", next))
    }

    fn project_mut<I, R>(&mut self, mut reference: Peekable<I>) -> Result<&mut Content>
    where
        I: Iterator<Item = R>,
        R: AsRef<str>,
    {
        let next_ = reference.next().ok_or(failed!(
            target: Release,
            "expected a field name, found nothing"
        ))?;
        let next = next_.as_ref();
        self.get_mut(next)?
            .content
            .project_mut(reference)
            .context(anyhow!("in a field named {}", next))
    }
}

impl Compile for ObjectContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, mut compiler: C) -> Result<Graph> {
        let object_node = self
            .iter()
            .map(|(name, field)| {
                compiler.build(name, &field.content).map(|graph| {
                    if field.optional {
                        KeyValueOrNothing::sometimes(name, graph)
                    } else {
                        KeyValueOrNothing::always(name, graph)
                    }
                })
            })
            .collect::<Result<ObjectNode>>()?;
        Ok(Graph::Object(object_node))
    }
}
