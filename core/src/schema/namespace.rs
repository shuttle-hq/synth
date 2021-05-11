use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{value::Value, Map};

use std::collections::BTreeMap;
use std::convert::AsRef;
use std::{default::Default, iter::FromIterator};

use super::inference::MergeStrategy;
use super::{suggest_closest, ArrayContent, Content, FieldRef, Find, Name};
use crate::compile::{Compile, Compiler};
use crate::graph::{Graph, KeyValueOrNothing};

#[allow(dead_code)]
type JsonObject = Map<String, Value>;

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct Namespace {
    #[serde(flatten)]
    pub collections: BTreeMap<Name, Content>,
}

impl AsRef<BTreeMap<Name, Content>> for Namespace {
    fn as_ref(&self) -> &BTreeMap<Name, Content> {
        &self.collections
    }
}

impl IntoIterator for Namespace {
    type Item = (Name, Content);

    type IntoIter = std::collections::btree_map::IntoIter<Name, Content>;

    fn into_iter(self) -> Self::IntoIter {
        self.collections.into_iter()
    }
}

impl FromIterator<(Name, Content)> for Namespace {
    fn from_iter<T: IntoIterator<Item = (Name, Content)>>(iter: T) -> Self {
        Self {
            collections: iter.into_iter().collect(),
        }
    }
}

impl Namespace {
    #[cfg(test)]
    pub fn accepts(&self, name: &Name, value: &Value) -> Result<()> {
        self.get_collection(name)?.accepts(value)
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&Name, &Content)> {
        self.collections.iter()
    }

    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = &Name> {
        self.collections.keys()
    }

    pub fn try_update<M: MergeStrategy<Content, Value>>(
        &mut self,
        strategy: M,
        name: &Name,
        value: &Value,
    ) -> Result<()> {
        let collection = self.get_collection_mut(name)?;
        strategy.try_merge(collection, value)?;
        Ok(())
    }

    pub fn collection_exists(&self, name: &Name) -> bool {
        self.collections.contains_key(name)
    }

    pub fn put_collection(&mut self, name: &Name, content: Content) -> Result<()> {
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

    pub fn collection(value: &Value) -> Content {
        Content::Array(ArrayContent {
            length: Box::new(Content::from(&Value::from(1))),
            content: Box::new(value.into()),
        })
    }

    pub fn create_collection(&mut self, name: &Name, value: &Value) -> Result<()> {
        let as_content = Self::collection(value);
        self.put_collection(name, as_content)?;
        Ok(())
    }

    pub fn delete_collection(&mut self, name: &Name) -> Result<()> {
        if self.collections.remove(name).is_none() {
            return Err(failed!(
                target: Release,
                "collection does not exist: {}",
                name
            ));
        }
        Ok(())
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.collections.is_empty()
    }

    // May remove this in due course. Or add an only visible for testing flag
    // or something like that.
    #[allow(dead_code)]
    pub fn export_schema(&self, name: &Name) -> Result<Content> {
        let schema = self.get_collection(name)?;
        Ok(schema.clone())
    }

    pub fn get_s_node_mut(&mut self, reference: &FieldRef) -> Result<&mut Content> {
        let collection = reference.collection();
        self.get_collection_mut(collection)?
            .find_mut(reference.iter_fields().peekable())
            .context(format!("in a collection: '{}'", collection))
    }

    pub fn get_s_node(&self, reference: &FieldRef) -> Result<&Content> {
        let collection = reference.collection();
        self.get_collection(collection)?
            .find(reference.iter_fields().peekable())
            .context(format!("in a collection: '{}'", collection))
    }

    pub fn get_collection_mut(&mut self, name: &Name) -> Result<&mut Content> {
        let suggest = suggest_closest(self.collections.keys(), name.as_ref()).unwrap_or_default();
        if let Some(collection) = self.collections.get_mut(name) {
            Ok(collection)
        } else {
            Err(failed!(target: Release, NotFound => "no such collection: '{}'{}", name, suggest))
        }
    }

    pub fn get_collection(&self, name: &Name) -> Result<&Content> {
        let suggest = suggest_closest(self.collections.keys(), name.as_ref()).unwrap_or_default();
        if let Some(collection) = self.collections.get(name) {
            Ok(collection)
        } else {
            Err(failed!(target: Release, NotFound => "no such collection: '{}'{}", name, suggest))
        }
    }
}

impl Compile for Namespace {
    fn compile<'a, C: Compiler<'a>>(&'a self, mut compiler: C) -> Result<Graph> {
        // TODO: needs to wrap each top-level attribute in a variable size array model
        let object_node = self
            .iter()
            .map(|(name, field)| {
                compiler
                    .build(name.as_ref(), field)
                    .map(|graph| KeyValueOrNothing::always(name.as_ref(), graph))
            })
            .collect::<Result<_>>()?;
        Ok(Graph::Object(object_node))
    }
}
