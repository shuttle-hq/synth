//! convert [`Content`](crate::schema::Content) to [`Graph`](crate::graph::Graph).
//!
//! This module contains all the logic for taking a [`Content`](crate::schema::Content)
//! to a [`Graph`](crate::graph::Graph). Where [`Content`](crate::schema::Content) is a user-facing
//! specification of generators, [`Graph`](crate::graph::Graph) is the core data structure used in
//! generating data. It is [`Graph`](crate::graph::Graph) that implements
//! [`Generator`](synth_gen::prelude::Generator).
//!
//! While [`Content`](crate::schema::Content) is a tree, [`Graph`](crate::graph::Graph) is a DAG.
//! Within a [`Graph`](crate::graph"::Graph), there may exist relations (encoded as `same_as` at
//! the level of [`Content`](crate::schema::Content)) which force certain generators to track the
//! values generated by others. This is done through wrapping these trackers in the [`Link`](Link)
//! type.
//!
//! The core of this module is the [`Compiler`](Compiler)/[`Compile`](Compile) pair of traits.
//! All [`Content`](crate::schema::Content) nodes implement [`Compile`](Compile) and get visited by
//! [`Compiler`](Compiler)s.
//!
//! The main structure of this module is [`NamespaceCompiler`](NamespaceCompiler), which can be used
//! to compile both [`Namespace`](crate::schema::Namespace) and [`Content`](crate::schema::Content)
//! into [`Graph`](crate::graph::Graph).

use std::collections::{BTreeMap, BTreeSet};
use std::iter::IntoIterator;

use anyhow::{Context, Result};

mod state;
use state::{CompilerState, OutputState, Artifact, Source, Symbols};

mod address;
use address::Address;

pub mod link;
pub use link::{FromLink, Link};
use link::{Recorder, Ordered, GeneratorSliceRef, GeneratorRecorder};

use crate::graph::Graph;
use crate::schema::{Content, Namespace};

/// A trait for visitors of the [`Content`](crate::schema::Content) tree.
///
/// Named children are visited (and built) by calling `build` and the value of a node at a
/// different location in the tree (such as that required by `same_as`) is obtained by calling
/// `get`.
pub trait Compiler<'a> {
    /// Build the child node called `field`.
    fn build(&mut self, field: &str, content: &'a Content) -> Result<Graph>;

    /// Access the built value of the node at address `field`.
    fn get<S: Into<Address>>(&mut self, field: S) -> Result<Graph>;
}

pub trait Compile {
    fn compile<'a, C: Compiler<'a>>(&'a self, compiler: C) -> Result<Graph>;
}

pub struct NamespaceCompiler<'a> {
    state: CompilerState<'a, Graph>,
    vtable: Symbols,
}

impl<'a> NamespaceCompiler<'a> {
    fn new_at(state: CompilerState<'a, Graph>) -> Self {
        let vtable = Symbols::new();
        Self { state, vtable }
    }

    pub fn new(namespace: &'a Namespace) -> Self {
        let state = CompilerState::namespace(namespace);
        Self::new_at(state)
    }

    pub fn new_flat(content: &'a Content) -> Self {
        let state = CompilerState::content(content);
        Self::new_at(state)
    }

    pub fn compile(mut self) -> Result<Graph> {
        let crawler = Crawler {
            state: &mut self.state,
            symbols: &mut self.vtable,
            position: Address::new_root(),
        };

        crawler.compile()?;

        let mut visits = vec![Address::new_root()];

        while let Some(address) = visits.pop() {
            debug!("{}", address);
            let next = {
                let state = self.state.project(address.clone())?;

                if state.output().is_built() {
                    continue;
                }

                let mut targets = BTreeSet::new();
                let mut pushbacks = BTreeSet::new();
                for target in state.refs().iter() {
                    // Direct references go first
                    let is_built = self.vtable
                        .is_built(&address, target)
                        .ok_or_else(|| anyhow!("undefined reference to `{}` from `{}`", target, address))?;
                    if !is_built {
                        debug!("direct reference to `{}`", target);
                        targets.insert(target.clone());
                    }

                    // Then pushbacks to first child of the common root
                    let (mut root, mut relative_to) = target.relativize(&address);
                    if !relative_to.is_root() {
                        let child = relative_to.deeper().unwrap();
                        root.at(&child);
                        let is_built = self.state
                            .project(root.clone())?
                            .output()
                            .is_built();
                        if !is_built && !relative_to.is_root() {
                            debug!("pushback dependency to `{}`", root);
                            pushbacks.insert(root);
                        }
                    }
                }

                // Finally, we push direct descendents
                let descents: BTreeSet<Address> = state
                    .scope()
                    .iter_keys()
                    .filter_map(|child| {
                        debug!("  {}", child);
                        let is_built = state.get(&child).unwrap().output().is_built();
                        let child_address = address.clone().into_at(&child);
                        if !is_built
                            && !targets.contains(&child_address)
                            && !pushbacks.contains(&child_address) {
                            Some(child_address)
                        } else {
                            None
                        }
                    })
                    .collect();

                targets
                    .into_iter()
                    .chain(pushbacks.into_iter())
                    .chain(descents.into_iter())
                    .collect::<Vec<_>>()
            };

            if !next.is_empty() {
                debug!("dependencies not satisfied: {:?}", next);
                if matches!(self.state.project_mut(address.clone())?.output_mut().waiting(), OutputState::Waiting) {
		    // This node was visited once and was waiting for dependencies to be built first,
		    // then is now being visited a second time so is a dependency of itself.
                    return Err(anyhow!("cycle detected at {}", address));
                }
                visits.push(address);
                visits.extend(next);
                continue;
            }

            if !address.is_root() {
                let mut parent = address.clone();
                let this = parent.shallower().unwrap();
                // It is not necessary to order nodes that are not bound to be wrapped in `Ordered`
                if self.vtable.contains(&parent) {
                    self.state.project_mut(parent)?.scope_mut().push(this).unwrap();
                }
            }

            let state = self.state.project_mut(address.clone())?;
            let vtable = &mut self.vtable;
            let mut children = BTreeMap::new();
            let content_compiler = ContentCompiler {
                scope: address.clone(),
                state,
                children: &mut children,
                vtable
            };

            let mut node = content_compiler
                .compile()
                .with_context(|| format!("while trying to build `{}`", &address))?;

            if let Some(local_table) = vtable.get(&address) {
                // `node` must be wrapped in `Ordered`
                let mut scope = local_table.values().map(|factory| {
                    factory.get_source().unwrap()
                }).collect::<Vec<_>>();
                let mut ordered_children = Vec::new();
                for child in state.scope().iter_ordered() {
                    let (recorder, slice_ref) = children.remove(child).unwrap();
                    scope.push(slice_ref);
                    ordered_children.push((child.to_string(), recorder));
                }
                for (child, (recorder, slice_ref)) in children.into_iter() {
                    scope.push(slice_ref);
                    ordered_children.push((child, recorder));
                }
                node = Graph::from_link(Link::Ordered(Ordered::new(scope, ordered_children, node)));
            }

            let artifact = if vtable.targetted(&address) {
                let recorder = Recorder::wrap(node);
                vtable.paths(&address).into_iter().try_for_each(|(root, tail)| {
                    debug!("setting source root=`{}` tail=`{}`", root, tail);
                    vtable.set_source(&root, &tail, recorder.new_slice())
                })?;
                Artifact::from_recorder(recorder)
            } else {
                Artifact::just(node)
            };

            state.output_mut().set_output(artifact);
        }

        Ok(self.state.move_output().unwrap().pack())
    }
}

pub struct ContentCompiler<'c, 'a: 'c> {
    scope: Address,
    state: &'c mut CompilerState<'a, Graph>,
    children: &'c mut BTreeMap<String, (GeneratorRecorder<Graph>, GeneratorSliceRef<Graph>)>,
    vtable: &'c mut Symbols,
}

impl<'c, 'a: 'c> ContentCompiler<'c, 'a> {
    fn compile(self) -> Result<Graph> {
        match self.state.source() {
            Source::Namespace(namespace) => namespace.compile(self),
            Source::Content(content) => content.compile(self),
        }
    }
}

impl<'c, 'a: 'c> Compiler<'a> for ContentCompiler<'c, 'a> {
    fn build(&mut self, field: &str, _: &'a Content) -> Result<Graph> {
        let mut child = self
            .state
            .get_mut(field)
            .expect("a previously visited child has disappeared")
            .move_output()
            .expect("a built child was not available");

        if self.vtable.contains(&self.scope) {
            // TODO: look into if we should use unpack here
            let recorder = Recorder::wrap(child.pack());
            let slice_ref = recorder.new_slice();
            let view = slice_ref.new_view();
            child = Artifact::from_view(view);
            self.children.insert(field.to_string(), (recorder, slice_ref));
        }

        Ok(child.pack())
    }

    fn get<S: Into<Address>>(&mut self, field: S) -> Result<Graph> {
        let address = field.into();
        let view = self.vtable.issue(&self.scope, &address)
            .with_context(|| anyhow!("while trying to access a reference to `{}` at `{}`", address, self.scope))?;
        Ok(Graph::from_link(Link::View(view)))
    }
}

pub struct Crawler<'t, 'a> {
    state: &'t mut CompilerState<'a, Graph>,
    symbols: &'t mut Symbols,
    position: Address,
}

impl<'t, 'a: 't> Crawler<'t, 'a> {
    fn as_at(&mut self, field: &str, content: &'a Content) -> Crawler<'_, 'a> {
        let position = self.position.clone().into_at(field);
        Crawler {
            state: self.state.entry(field).or_init(content),
            symbols: self.symbols,
            position,
        }
    }

    fn compile(self) -> Result<()> {
        match self.state.source() {
            Source::Namespace(namespace) => namespace.compile(self)?,
            Source::Content(content) => content.compile(self)?,
        };
        Ok(())
    }
}

impl<'t, 'a: 't> Compiler<'a> for Crawler<'t, 'a> {
    fn build(&mut self, field: &str, content: &'a Content) -> Result<Graph> {
        if let Err(err) = self.as_at(field, content).compile() {
            warn!("could not crawl into field `{}` at `{}`", field, self.position);
            return Err(err);
        }
        Ok(Graph::dummy())
    }

    fn get<S: Into<Address>>(&mut self, target: S) -> Result<Graph> {
        let target: Address = target.into();
        self.symbols.declare(self.position.clone(), target.clone())?;
        self.state.refs_mut().insert(target);
        Ok(Graph::dummy())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::tests::complete;
    use crate::graph::Value;

    use synth_gen::prelude::*;

    #[test]
    fn compile_no_ref() {
        let generator = generator!({
            "type": "bool",
            "constant": true
        });
        assert!(complete(generator).unwrap().as_bool().unwrap())
    }

    #[test]
    fn compile_linear() {
        let generator = generator!({
            "type": "object",
            "0": {
                "type": "bool",
                "constant": true
            },
            "1": "@0",
            "2": "@1",
            "3": "@2"
        });
        let value = complete(generator).unwrap();
        let as_object = value.as_object().unwrap();
        for i in 0..4 {
            assert!(as_object.get(&i.to_string()).unwrap().as_bool().unwrap())
        }
    }

    #[test]
    fn compile_deep_linear() {
        let generator = generator!({
            "type": "object",
            "0": {
                "type": "object",
                "0": {
                    "type": "bool",
                    "constant": true,
                }
            },
            "1": {
                "type": "object",
                "0": "@0.0"
            },
            "2": {
                "type": "object",
                "0": "@1.0"
            },
            "3": {
                "type": "object",
                "0": "@2.0"
            },
            "4": {
                "type": "object",
                "0": "@3.0"
            }
        });
        let value = complete(generator).unwrap();
        let as_object = value.as_object().unwrap();
        for i in 0..5 {
            assert!(as_object.get(&i.to_string()).unwrap().as_object().unwrap().get("0").unwrap().as_bool().unwrap())
        }
    }

    #[test]
    fn compile_circle() {
        let generator = try_generator!({
            "type": "object",
            "0": "@3",
            "1": "@0",
            "2": "@1",
            "3": "@2"
        });
        assert!(generator.is_err())
    }

    #[test]
    fn compile_deep_cycle() {
        let generator = try_generator!({
            "type": "object",
            "0": {
                "type": "object",
                "0": "@4.0"
            },
            "1": {
                "type": "object",
                "0": "@0.0"
            },
            "2": {
                "type": "object",
                "0": "@1.0"
            },
            "3": {
                "type": "object",
                "0": "@2.0"
            },
            "4": {
                "type": "object",
                "0": "@3.0"
            }
        });
        assert!(generator.is_err());
    }

    #[test]
    fn compile_nested_cycle() {
        let generator = try_generator!({
            "type": "object",
            "0": {
                "type": "object",
                "00": {
                    "type": "object",
                    "000": {
                        "type": "object",
                        "0000": "@1"
                    },
                    "001": "@0.00.000.0000"
                },
                "01": "@0.00.001"
            },
            "1": "@0.01"
        });
        assert!(generator.is_err())
    }

    #[test]
    fn compile_nested_linear() {
        let generator = generator!({
            "type": "object",
            "0": {
                "type": "object",
                "00": {
                    "type": "object",
                    "000": {
                        "type": "object",
                        "0000": {
                            "type": "bool",
                            "constant": true
                        }
                    },
                    "001": "@0.00.000.0000"
                },
                "01": "@0.00.001"
            },
            "1": "@0.01"
        });
        assert!(complete(generator).unwrap().as_object().unwrap().get("1").unwrap().as_bool().unwrap())
    }

    #[test]
    fn compile_star() {
        let generator = generator!({
            "type": "object",
            "0": {
                "type": "number",
                "subtype": "u64",
                "id": {}
            },
            "1": "@0",
            "2": "@0",
            "3": "@0",
            "4": {
                "type": "array",
                "length": 42,
                "content": "@0"
            }
        });
        let value = complete(generator).unwrap();
        let as_object = value.as_object().unwrap();
        for i in 0..3 {
            let number = as_object.get(&i.to_string()).unwrap().as_number().unwrap();
            assert!(matches!(number, Number::U64(1)));
        }
        assert!(as_object.get("4").unwrap().as_array().unwrap().iter().all(|v| {
            matches!(v.as_number().unwrap(), Number::U64(1))
        }))
    }

    #[test]
    fn compile_slices() {
        let generator = generator!({
            "type": "object",
            "0": {
                "type": "array",
                "length": 4,
                "content": {
                    "type": "number",
                    "subtype": "u64",
                    "id": {}
                }
            },
            "1": "@0.content",
            "2": {
                "type": "array",
                "length": 4,
                "content": "@0.content"
            },
            "22": "@2.content",
            "3": {
                "type": "array",
                "length": 2,
                "content": {
                    "type": "array",
                    "length": 2,
                    "content": "@0.content"
                }
            },
            "33": {
                "type": "array",
                "length": 4,
                "content": {
                    "type": "array",
                    "length": "@3.content.content",
                    "content": "@3.content.content"
                }
            }
        });
        let value = complete(generator).unwrap();
        let as_object = value.as_object().unwrap();

        let two = as_object.get("2").unwrap().as_array().unwrap();
        for (i, v) in as_object.get("0").unwrap().as_array().unwrap().iter().enumerate() {
            let ii = (i + 1) as u64;
            assert_eq!(*v, Value::Number(Number::U64(ii)));
            assert_eq!(*two.get(i).unwrap(), Value::Number(Number::U64(ii)));
        }

        assert!(matches!(as_object.get("1").unwrap().as_number().unwrap(), Number::U64(1)));

        assert!(matches!(as_object.get("22").unwrap().as_number().unwrap(), Number::U64(1)));

        let three = as_object.get("3").unwrap().as_array().unwrap();
        let mut idx = 0;
        for i in 0..2 {
            let matches = three.get(i).unwrap().as_array().unwrap().iter().all(|v| {
                idx += 1;
                *v == Value::Number(Number::U64(idx))
            });
            assert!(matches);
        }

        let three_three = as_object.get("33").unwrap().as_array().unwrap();
        let mut inner = (1..5).cycle();
        for (i, v) in three_three.iter().enumerate() {
            let v = v.as_array().unwrap();
            assert_eq!(v.len(), i + 1);
            v.iter().for_each(|vv| {
                assert_eq!(*vv, Value::Number(Number::U64(inner.next().unwrap())));
            });
        }
    }
}
