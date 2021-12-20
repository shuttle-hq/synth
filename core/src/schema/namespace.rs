use super::inference::MergeStrategy;
use super::{suggest_closest, Content, FieldRef, Find};
use crate::compile::{Compile, Compiler};
use crate::graph::prelude::OptionalMergeStrategy;
use crate::graph::{Graph, KeyValueOrNothing};

use std::collections::BTreeMap;
use std::{default::Default, iter::FromIterator};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{value::Value, Map};

#[allow(dead_code)]
type JsonObject = Map<String, Value>;

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Hash)]
pub struct Namespace {
    #[serde(flatten)]
    collections: BTreeMap<String, Content>,
}

impl IntoIterator for Namespace {
    type Item = (String, Content);

    type IntoIter = std::collections::btree_map::IntoIter<String, Content>;

    fn into_iter(self) -> Self::IntoIter {
        self.collections.into_iter()
    }
}

// TODO: Could allow entering invalid collection names.
impl FromIterator<(String, Content)> for Namespace {
    fn from_iter<T: IntoIterator<Item = (String, Content)>>(iter: T) -> Self {
        Self {
            collections: iter.into_iter().collect(),
        }
    }
}

impl Namespace {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub fn accepts(&self, name: &str, value: &Value) -> Result<()> {
        self.get_collection(name)?.accepts(value)
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Content)> {
        self.collections.iter().map(|(k, v)| (k.as_str(), v))
    }

    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.collections.keys().map(String::as_str)
    }

    pub fn default_try_update(&mut self, name: &str, value: &Value) -> Result<()> {
        self.try_update(OptionalMergeStrategy, name, value)
    }

    pub fn try_update<M: MergeStrategy<Content, Value>>(
        &mut self,
        strategy: M,
        name: &str,
        value: &Value,
    ) -> Result<()> {
        let collection = self.get_collection_mut(name)?;
        strategy.try_merge(collection, value)?;
        Ok(())
    }

    pub fn collection_exists(&self, name: &str) -> bool {
        self.collections.contains_key(name)
    }

    pub fn put_collection(&mut self, name: String, content: Content) -> Result<()> {
        super::check_collection_name_is_valid(&name)?;

        if self.collections.insert(name.clone(), content).is_some() {
            Err(failed!(
                target: Release,
                "collection already exists: {}",
                name
            ))
        } else {
            Ok(())
        }
    }

    pub fn put_collection_from_json(&mut self, name: String, value: &Value) -> Result<()> {
        self.put_collection(name, Content::from_value_wrapped_in_array(value))
    }

    pub fn remove_collection(&mut self, name: &str) -> Option<Content> {
        self.collections.remove(name)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.collections.is_empty()
    }

    pub fn len(&self) -> usize {
        self.collections.len()
    }

    // May remove this in due course. Or add an only visible for testing flag
    // or something like that.
    #[allow(dead_code)]
    pub fn export_schema(&self, name: &str) -> Result<Content> {
        let schema = self.get_collection(name)?;
        Ok(schema.clone())
    }

    pub fn get_s_node_mut(&mut self, reference: &FieldRef) -> Result<&mut Content> {
        let collection = reference.collection();
        self.get_collection_mut(collection)?
            .find_mut(reference.iter_fields().peekable())
            .with_context(|| format!("in a collection: '{}'", collection))
    }

    pub fn get_s_node(&self, reference: &FieldRef) -> Result<&Content> {
        let collection = reference.collection();
        self.get_collection(collection)?
            .find(reference.iter_fields().peekable())
            .with_context(|| format!("in a collection: '{}'", collection))
    }

    pub fn get_collection_mut(&mut self, name: &str) -> Result<&mut Content> {
        let suggest = suggest_closest(self.collections.keys(), name).unwrap_or_default();
        if let Some(collection) = self.collections.get_mut(name) {
            Ok(collection)
        } else {
            Err(failed!(target: Release, NotFound => "no such collection: '{}'{}", name, suggest))
        }
    }

    pub fn get_collection(&self, name: &str) -> Result<&Content> {
        let suggest = suggest_closest(self.collections.keys(), name).unwrap_or_default();
        if let Some(collection) = self.collections.get(name) {
            Ok(collection)
        } else {
            Err(failed!(target: Release, NotFound => "no such collection: '{}'{}", name, suggest))
        }
    }
}

impl Compile for Namespace {
    fn compile<'a, C: Compiler<'a>>(&'a self, mut compiler: C) -> Result<Graph> {
        let object_node = self
            .iter()
            .map(|(name, content)| {
                compiler
                    .build(name, content)
                    .map(|graph| KeyValueOrNothing::always(name, graph, false))
            })
            .collect::<Result<_>>()?;
        Ok(Graph::Object(object_node))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::NullContent;

    #[test]
    fn check_name_valid_on_collection_insert() {
        let mut ns = Namespace::new();

        assert!(ns
            .put_collection("世界".to_string(), Content::Null(NullContent))
            .is_err());

        assert!(ns
            .put_collection_from_json("!!!".to_string(), &Value::Null)
            .is_err());
    }
}
