use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{value::Value, Map};

use std::collections::HashMap;
use std::convert::AsRef;
use std::{default::Default, iter::FromIterator};

use super::inference::{MergeStrategy, OptionalMergeStrategy};
use super::{Name, suggest_closest, ArrayContent, Content, FieldRef, Find};
use crate::gen::{Compile, Compiler, Model};

use synth_gen::{value::IntoToken, Chain, TokenGeneratorExt};

#[allow(dead_code)]
type JsonObject = Map<String, Value>;

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct Namespace {
    #[serde(flatten)]
    pub collections: HashMap<Name, Content>,
}

impl AsRef<HashMap<Name, Content>> for Namespace {
    fn as_ref(&self) -> &HashMap<Name, Content> {
        &self.collections
    }
}

impl IntoIterator for Namespace {
    type Item = (Name, Content);

    type IntoIter = std::collections::hash_map::IntoIter<Name, Content>;

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

    pub fn try_update(&mut self, name: &Name, value: &Value) -> Result<()> {
        let collection = self.get_collection_mut(name)?;
        // TODO put inference_strategy in the namespace engine
        OptionalMergeStrategy.try_merge(collection, value)?;
        Ok(())
    }

    pub fn collection_exists(&self, name: &Name) -> bool {
        self.collections.contains_key(name)
    }

    pub fn create_collection(&mut self, name: &Name, value: &Value) -> Result<()> {
        let as_content = Content::Array(ArrayContent {
            length: Box::new(Content::from(&Value::from(1))),
            content: Box::new(value.into()),
        });
        if let Some(_) = self.collections.insert(name.clone(), as_content) {
            return Err(failed!(
                target: Release,
                "collection already exists: {}",
                name
            ));
        }
        Ok(())
    }

    pub fn delete_collection(&mut self, name: &Name) -> Result<()> {
        if let None = self.collections.remove(name) {
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

    fn get_collection_mut(&mut self, name: &Name) -> Result<&mut Content> {
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
    fn compile<'a, C: Compiler<'a>>(&'a self, mut compiler: C) -> Result<Model> {
        // TODO: needs to wrap each top-level attribute in a variable size array model
        let generator = self
            .iter()
            .map(|(name, content)| {
                compiler
                    .build(name.as_ref(), content)
                    .map(|value| value.with_key(name.to_string().yield_token()))
            })
            .collect::<Result<Chain<_>>>()?
            .into_map(None);
        Ok(Model::Object(generator))
    }
}
