use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{value::Value, Map};
use std::collections::BTreeMap;
use std::convert::AsRef;
use std::{default::Default, iter::FromIterator};
use super::content::{FieldContent, ObjectContent, OneOfContent, StringContent, FormatContent};
use super::inference::MergeStrategy;
use super::{suggest_closest, ArrayContent, Content, FieldRef, Find, Name};
use crate::compile::{Compile, Compiler};
use crate::graph::prelude::OptionalMergeStrategy;
use crate::graph::{Graph, KeyValueOrNothing};

use std::collections::{HashMap, VecDeque};
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
            .with_context(|| format!("in a collection: '{}'", collection))
    }

    pub fn get_s_node(&self, reference: &FieldRef) -> Result<&Content> {
        let collection = reference.collection();
        self.get_collection(collection)?
            .find(reference.iter_fields().peekable())
            .with_context(|| format!("in a collection: '{}'", collection))
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
    fn get_adj_list(&self) -> Vec<(Name, Name)> {
        let mut list = Vec::new();
        for (n,c) in self.collections.iter() {
            get_list((n,c), &mut list);
        };
        list
    }

    pub fn topo_sort(&self) -> Option<Vec<Name>> {
        let lists: Vec<(Name, Name)> = self.get_adj_list();
        let mut q: VecDeque<Name> = VecDeque::new();
        let mut sorted: Vec<Name> = Vec::new();
        let mut graph: NameGraph = NameGraph::new();
        for v in &lists {
            graph.entry(v.0.clone()).or_insert_with(Vec::new).push(v.1.clone());
        };
        log::info!("lists: {:?}", lists);
        let mut in_degrees: BTreeMap<Name, usize> = BTreeMap::new();

        for (p,c) in &lists {
            *in_degrees.entry(c.clone()).or_insert(0) += 1;
            in_degrees.entry(p.clone()).or_insert(0);
        };

        for n in &in_degrees {
            if n.1 == &0 {
                q.push_back(n.0.clone());
            }
        }
        
        while let Some(name) = q.pop_front() {
            sorted.push(name.clone());
            log::info!("name: {:?}", name);
            if graph.contains_key(&name) {
                for out in graph.index(&name) {
                    in_degrees.entry(out.clone()).and_modify(|v| *v -= 1);
                    if in_degrees.iter().any(|v| v.1 == &0usize) {
                        q.push_back(out.clone());
                    }
                }
            };
        }
        
        if sorted.len() == in_degrees.keys().len() {
            log::info!("{:?}", sorted);
            //if the name has no same_as type field, it must still be in the list, but may come in any order, back or front
            for name in self.collections.keys() {
                if !sorted.contains(name) {
                    sorted.push(name.clone());
                }
            };
            Some(sorted)
        } else {
            None
        }
    }
}

fn get_list((n,c):(&Name, &Content), list: &mut Vec<(Name, Name)>) {
            match c {
                Content::Object(ObjectContent{fields}) => {
                    for cont in fields.values() {
                        let FieldContent {box content,.. } = cont;
                        if let Content::SameAs(same) = content {
                            let (l,r) = (same.ref_.collection().clone(), n.clone());
                            if l != r  && !list.contains(&(l.clone(),r.clone())) {
                                list.push((l,r));
                            };
                            
                        } else {
                            get_list((n,content), list);
                        }
                    };
                }
                Content::Array(ArrayContent {box content,.. }) => {
                    match content.clone() {
                        Content::Object(obj) => {
                            get_list((n,&Content::Object(obj)), list);
                        }
                        Content::Array(arr) => {
                            get_list((n,&Content::Array(arr)), list);
                        }
                        Content::SameAs(same) => {
                            get_list((n,&Content::SameAs(same)), list);
                        }
                        Content::OneOf(one) => {
                            get_list((n,&Content::OneOf(one)), list);
                        }
                        Content::String(s) => {
                            get_list((n,&Content::String(s)), list);
                        }
                        _ => {}

                    }
                }
                Content::OneOf(OneOfContent{variants}) => {
                    for variant in variants {
                        let box variant = variant.content.clone();
                        match  variant {
                            Content::Array(arr) => {
                                get_list((n,&Content::Array(arr)), list);
                            }
                            Content::Object(obj) => {
                                get_list((n,&Content::Object(obj)), list);
                            }
                            Content::OneOf(one) => {
                                get_list((n,&Content::OneOf(one)), list);
                            }
                            Content::SameAs(same) => {
                                get_list((n,&Content::SameAs(same)), list);
                            }
                            Content::String(s) => {
                                get_list((n,&Content::String(s)), list);
                            }
                            _ => {}
                        }
                    }
                }
                Content::SameAs(same_as_content) => {
                let pair = (same_as_content.ref_.collection().clone(), n.clone());
                if pair.0 != pair.1 && !list.contains(&pair) {
                    list.push(pair);
                };
                }
                Content::String(StringContent::Format(form)) => {
                    //the only variant of string that can hold a content
                    let FormatContent{ref arguments, ..} = form;
                    for (_, c) in arguments.iter() {
                        get_list((n,c), list);
                    } 
                }
                _ => {}
            }
    
}
impl Compile for Namespace {
    fn compile<'a, C: Compiler<'a>>(&'a self, mut compiler: C) -> Result<Graph> {
        // TODO: needs to wrap each top-level attribute in a variable size array model
        let sorted_ns = self.topo_sort().ok_or_else(|| anyhow!("dependency is cyclic"))?;
        let object_node = sorted_ns.iter().map(|name|{
            let field = self.collections.get(name).expect("field should be there");
            compiler.build(name.as_ref(), field).map(|graph| KeyValueOrNothing::always(name.as_ref(), graph))
        }).collect::<Result<_>>()?;
        Ok(Graph::Object(object_node))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::Content;
    use crate::schema::BoolContent;
    #[test]
    fn test_sort_simple() {
        let mut namespace = Namespace {
            collections: BTreeMap::new(),
        };
        let ref1: FieldRef = "visitors.address.postcode".parse().unwrap();
        let ref2: FieldRef = "daughters.address.postcode".parse().unwrap();

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
        namespace.put_collection(&"visitors".parse::<Name>().unwrap(), Content::Bool(BoolContent::Constant(true))).unwrap();
        namespace.put_collection(&"daughters".parse::<Name>().unwrap(),Content::Bool(BoolContent::Constant(false) )).unwrap();
        println!("sorted: {:?}", namespace.topo_sort());
        let sorted = namespace.topo_sort().unwrap();
        let length = sorted.len();
        for i in 0..length {
            assert!(check_dep(&sorted[..i], &namespace))
        }
    }

    #[test]
    fn test_sort_complex() {
        let mut namespace = Namespace {
            collections: BTreeMap::new(),
        };
        let ref1: FieldRef = "visitors.address.postcode".parse().unwrap();

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
        println!("sorted: {:?}", namespace.topo_sort());
        namespace.put_collection(&"visitors".parse::<Name>().unwrap(), Content::Bool(BoolContent::Constant(true))).unwrap();
        namespace.put_collection(&"daughters".parse::<Name>().unwrap(),Content::Bool(BoolContent::Constant(false) )).unwrap();
        
        println!("sorted: {:?}", namespace.topo_sort());
        let sorted = namespace.topo_sort().unwrap();
        let length = sorted.len();
        for i in 0..length {
            assert!(check_dep(&sorted[..i], &namespace))
        }
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
        println!("sorted: {:?}", namespace.topo_sort());
        assert!(namespace.topo_sort().is_none());
    }

    //helper method for checking sorted dependencies
    fn check_dep(list: &[Name], ns: &Namespace) -> bool {
        let (last, list)= match list.split_last() {
            Some(n) => n,
            None=> return true
        };
        let c = ns.get_collection(last).unwrap();
        if let Content::SameAs(same) = c {
            list.contains(same.ref_.collection()) && check_dep(list, ns)
        } else {
            true
        }
    }
}
