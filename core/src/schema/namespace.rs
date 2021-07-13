use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{value::Value, Map};

use std::collections::BTreeMap;
use std::convert::AsRef;
use std::{default::Default, iter::FromIterator};

use super::inference::MergeStrategy;
use super::{suggest_closest, ArrayContent, Content, FieldRef, Find, Name};
use crate::compile::{Compile, Compiler};
use crate::graph::prelude::OptionalMergeStrategy;
use crate::graph::{Graph, KeyValueOrNothing};

use std::collections::{HashMap, HashSet, VecDeque};
use std::ops::Index;

#[allow(dead_code)]
type JsonObject = Map<String, Value>;

//graph alies
type NameGraph = HashMap<Name, Vec<Name>>;

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

    pub fn default_try_update(&mut self, name: &Name, value: &Value) -> Result<()> {
        self.try_update(OptionalMergeStrategy, name, value)
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
    fn get_ajd_list(&self) -> Vec<(Name, Name)> {
        let mut list = Vec::new();
        for (n, c) in self.clone() {
            match c {
                Content::SameAs(same_as_content) => {
                    list.push((same_as_content.ref_.collection().clone(), n));
                }
                _ => {}
            }
        }
        list
    }

    pub fn topo_sort(&self) -> Option<Vec<Name>> {
        eprintln!("namespace: {:?}", self);
        let lists: Vec<(Name, Name)> = self.get_ajd_list();
        let mut q: VecDeque<Name> = VecDeque::new();
        //topologically sorted vec of names to be returned
        let mut sorted: Vec<Name> = Vec::new();

        //or graph
        let mut graph: NameGraph = HashMap::new();
        //make a slice of lists
        let dep_lists = lists.get(0..).unwrap();
        //here we build our graph by running through deplists
        for v in dep_lists {
            let source = &mut graph.entry(v.0.clone()).or_insert_with(|| Vec::new());
            source.push(v.1.clone());
        }
        eprintln!("dep_lists: {:?}", dep_lists);
        //kahn's algorithm says we need a way to count the inward edes into a node
        let mut in_degrees: BTreeMap<Name, usize> = BTreeMap::new();
        //get a set of all the names. we'll need it to develop our indegree and outedges
        let set: HashSet<Name> = lists
            .iter()
            .cloned()
            .map(|(p, c)| vec![p, c])
            .flatten()
            .collect();
        for name in &set {
            let entry = in_degrees.entry(name.clone()).or_insert(0);
            *entry = lists.iter().filter(|(_, c)| name == c).count();
        }
        eprintln!("indegrees: {:?}", in_degrees);
        eprintln!("set: {:?}", set);
        eprintln!("graph: {:?}", graph);
        //first push all the names with in_degree of zero into the queue
        for n in &in_degrees {
            if n.1 == &0 {
                q.push_back(n.0.clone());
            }
        }
        //then we loop through the queue
        while let Some(name) = q.pop_front() {
            sorted.push(name.clone());
            eprintln!("name: {:?}", name);
            if graph.contains_key(&name) {
                for out in graph.index(&name) {
                    in_degrees.entry(out.clone()).and_modify(|v| *v -= 1);
                    if in_degrees.iter().find(|v| v.1 == &0usize).is_some() {
                        q.push_back(out.clone());
                    }
                }
            };
        }
        if sorted.len() == set.len() {
            println!("{:?}", sorted.clone());
            Some(sorted)
        } else {
            None
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sort_simple() {
        let mut namespace = Namespace {
            collections: BTreeMap::new(),
        };
        let ref1: FieldRef = "visitors.address.postcode".parse().unwrap();
        let ref2: FieldRef = "daughters.address.postcode".parse().unwrap();
        //            let ref3: FieldRef = "winners.address.postcode".parse().unwrap();

        namespace
            .put_collection(
                &"users".parse::<Name>().unwrap(),
                Content::SameAs(crate::schema::SameAsContent { ref_: ref1 }),
            )
            .unwrap();
        namespace
            .put_collection(
                &"sons".parse::<Name>().unwrap(),
                Content::SameAs(crate::schema::SameAsContent { ref_: ref2 }),
            )
            .unwrap();
        // namespace.put_collection(
        //     &"losers".parse::<Name>().unwrap(),
        //     Content::SameAs(crate::schema::SameAsContent { ref_: ref3 }),
        // );
        println!("sorted: {:?}", namespace.topo_sort());
        assert!(
            namespace.topo_sort().unwrap()
                == (vec![
                    "visitors".parse::<Name>().unwrap(),
                    "daughters".parse::<Name>().unwrap(),
                    "users".parse::<Name>().unwrap(),
                    "sons".parse::<Name>().unwrap(),
                ])
                || namespace.topo_sort().unwrap()
                    == (vec![
                        "daughters".parse::<Name>().unwrap(),
                        "visitors".parse::<Name>().unwrap(),
                        "users".parse::<Name>().unwrap(),
                        "sons".parse::<Name>().unwrap(),
                    ])
                || namespace.topo_sort().unwrap()
                    == (vec![
                        "visitors".parse::<Name>().unwrap(),
                        "daughters".parse::<Name>().unwrap(),
                        "sons".parse::<Name>().unwrap(),
                        "users".parse::<Name>().unwrap(),
                    ])
                || namespace.topo_sort().unwrap()
                    == (vec![
                        "daughters".parse::<Name>().unwrap(),
                        "visitors".parse::<Name>().unwrap(),
                        "sons".parse::<Name>().unwrap(),
                        "users".parse::<Name>().unwrap(),
                    ])
        );
    }
    #[test]
    fn test_sort_complex() {
        let mut namespace = Namespace {
            collections: BTreeMap::new(),
        };
        let ref1: FieldRef = "visitors.address.postcode".parse().unwrap();
        //let ref2: FieldRef = "daughters.address.postcode".parse().unwrap();
        //            let ref3: FieldRef = "winners.address.postcode".parse().unwrap();

        namespace
            .put_collection(
                &"users".parse::<Name>().unwrap(),
                Content::SameAs(crate::schema::SameAsContent { ref_: ref1.clone() }),
            )
            .unwrap();
        namespace
            .put_collection(
                &"sons".parse::<Name>().unwrap(),
                Content::SameAs(crate::schema::SameAsContent { ref_: ref1 }),
            )
            .unwrap();
        // namespace.put_collection(
        //     &"losers".parse::<Name>().unwrap(),
        //     Content::SameAs(crate::schema::SameAsContent { ref_: ref3 }),
        // );
        println!("sorted: {:?}", namespace.topo_sort());
        assert!(
            namespace.topo_sort().unwrap()
                == (vec![
                    "visitors".parse::<Name>().unwrap(),
                    //"daughters".parse::<Name>().unwrap(),
                    "users".parse::<Name>().unwrap(),
                    "sons".parse::<Name>().unwrap(),
                ])
                || namespace.topo_sort().unwrap()
                    == (vec![
                        "visitors".parse::<Name>().unwrap(),
                        //"daughters".parse::<Name>().unwrap(),
                        "sons".parse::<Name>().unwrap(),
                        "users".parse::<Name>().unwrap(),
                    ])
        );
    }

    #[test]
    fn test_sort_cycle() {
        let mut namespace = Namespace {
            collections: BTreeMap::new(),
        };
        let ref1: FieldRef = "visitors.address.postcode".parse().unwrap();
        let ref2: FieldRef = "users.address.postcode".parse().unwrap();
        //            let ref3: FieldRef = "winners.address.postcode".parse().unwrap();

        namespace
            .put_collection(
                &"users".parse::<Name>().unwrap(),
                Content::SameAs(crate::schema::SameAsContent { ref_: ref1 }),
            )
            .unwrap();
        namespace
            .put_collection(
                &"visitors".parse::<Name>().unwrap(),
                Content::SameAs(crate::schema::SameAsContent { ref_: ref2 }),
            )
            .unwrap();
        // namespace.put_collection(
        //     &"losers".parse::<Name>().unwrap(),
        //     Content::SameAs(crate::schema::SameAsContent { ref_: ref3 }),
        // );
        println!("sorted: {:?}", namespace.topo_sort());
        assert!(namespace.topo_sort() == None);
    }
}
