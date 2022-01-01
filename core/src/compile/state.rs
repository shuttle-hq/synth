use anyhow::Result;

use std::collections::{BTreeMap, BTreeSet};
use std::iter::FromIterator;

use synth_gen::prelude::*;

use super::link::{Recorder, SliceRef, TapeView};
use super::{Address, Compile, Compiler, FromLink, Link};

use crate::{Content, Graph};

/// A holder struct for the compiler's internal state of the children of a given node.
///
/// The collection of children can be ordered by the compiler - though not all of them need to be.
/// The ordering is typically set as a result of dependencies between nodes.
pub(super) struct StructuredState<'a, G: Generator> {
    children: BTreeMap<String, CompilerState<'a, G>>,
    ordering: Vec<String>,
}

#[allow(dead_code)]
impl<'a, G: Generator> StructuredState<'a, G> {
    #[inline]
    pub(super) fn get(&self, field: &str) -> Option<&CompilerState<'a, G>> {
        self.children.get(field)
    }

    #[inline]
    pub(super) fn get_mut(&mut self, field: &str) -> Option<&mut CompilerState<'a, G>> {
        self.children.get_mut(field)
    }

    pub(super) fn iter_keys(&self) -> impl Iterator<Item = String> {
        let mut keys = self.children.keys().cloned().collect::<BTreeSet<_>>();
        let mut ordered = Vec::new();
        self.ordering.iter().for_each(|child| {
            keys.remove(child);
            ordered.push(child.clone());
        });
        ordered.into_iter().chain(keys.into_iter())
    }

    pub(super) fn iter_ordered(&self) -> std::slice::Iter<String> {
        self.ordering.iter()
    }

    #[inline]
    pub(super) fn push(&mut self, key: String) -> Option<()> {
        self.children.get(&key)?;
        self.ordering.push(key);
        Some(())
    }

    #[inline]
    pub(super) fn iter_values(&self) -> impl Iterator<Item = &CompilerState<'a, G>> {
        self.iter_keys()
            .map(move |key| self.children.get(&key).unwrap())
    }

    #[inline]
    pub(super) fn iter(&self) -> impl Iterator<Item = (String, &CompilerState<'a, G>)> {
        self.iter_keys().map(move |key| {
            let value = self.get(&key).unwrap();
            (key, value)
        })
    }

    pub(super) fn insert(
        &mut self,
        name: String,
        state: CompilerState<'a, G>,
    ) -> Option<CompilerState<'a, G>> {
        self.children.insert(name, state)
    }
}

impl<'a, G: Generator> FromIterator<(String, CompilerState<'a, G>)> for StructuredState<'a, G> {
    fn from_iter<I: IntoIterator<Item = (String, CompilerState<'a, G>)>>(iter: I) -> Self {
        let (ordering, children) = iter
            .into_iter()
            .map(|(name, state)| (name.clone(), (name, state)))
            .unzip();
        Self { children, ordering }
    }
}

impl<'a, G: Generator> IntoIterator for StructuredState<'a, G> {
    type Item = (String, CompilerState<'a, G>);
    type IntoIter = IntoIter<'a, G>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::from(self)
    }
}

impl<'a, G: Generator> std::iter::Extend<(String, CompilerState<'a, G>)>
    for StructuredState<'a, G>
{
    fn extend<T: IntoIterator<Item = (String, CompilerState<'a, G>)>>(&mut self, iter: T) {
        for (name, state) in iter {
            self.insert(name, state);
        }
    }
}

impl<'a, G: Generator> Default for StructuredState<'a, G> {
    fn default() -> Self {
        Self {
            children: BTreeMap::new(),
            ordering: Vec::new(),
        }
    }
}

pub(super) struct IntoIter<'a, G: Generator> {
    children: BTreeMap<String, CompilerState<'a, G>>,
    iter: std::vec::IntoIter<String>,
}

impl<'a, G: Generator> From<StructuredState<'a, G>> for IntoIter<'a, G> {
    #[inline]
    fn from(v: StructuredState<'a, G>) -> Self {
        Self {
            children: v.children,
            iter: v.ordering.into_iter(),
        }
    }
}

impl<'a, G: Generator> Iterator for IntoIter<'a, G> {
    type Item = (String, CompilerState<'a, G>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .and_then(|name| self.children.remove(&name).map(|state| (name, state)))
    }
}

pub enum Source<'a> {
    Namespace(&'a Content),
    Collection(&'a Content),
}

impl<'a> std::fmt::Display for Source<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Namespace(_) => write!(f, "namespace"),
            Self::Collection(_) => write!(f, "schema"),
        }
    }
}

impl Compile for Source<'_> {
    fn compile<'a, C: Compiler<'a>>(&'a self, compiler: C) -> Result<Graph> {
        match self {
            Source::Collection(schema) => schema.compile(compiler),
            Source::Namespace(ns) => ns.compile(compiler),
        }
    }
}

#[allow(dead_code)]
impl<'a> Source<'a> {
    #[inline]
    pub(super) fn as_namespace(&self) -> Result<&'a Content> {
        match self {
            Self::Namespace(namespace) => Ok(namespace),
            other => Err(failed!(
                target: Release,
                "source node type mismatch: expected namespace, found {}",
                other
            )),
        }
    }

    #[inline]
    pub(super) fn as_schema(&self) -> Result<&'a Content> {
        match self {
            Self::Collection(content) => Ok(content),
            other => Err(failed!(
                target: Release,
                "source node type mismatch: expected schema, found {}",
                other
            )),
        }
    }
}

pub(super) enum Artifact<G, Y, R> {
    Just(G),
    Link(Link<G, Y, R>),
}

impl<G, Y, R> Artifact<G, Y, R>
where
    G: Generator<Yield = Y, Return = R> + FromLink<Yield = Y, Return = R>,
{
    pub(super) fn pack(self) -> G {
        match self {
            Self::Just(g) => g,
            Self::Link(link) => G::from_link(link),
        }
    }

    pub(super) fn just(inner: G) -> Self {
        Self::Just(inner)
    }

    pub(super) fn from_view(view: TapeView<Y, R>) -> Self {
        Self::Link(Link::View(view))
    }

    pub(super) fn from_recorder(recorder: Recorder<G, Y, R>) -> Self {
        Self::Link(Link::Recorder(recorder))
    }
}

pub(super) type GenArtifact<G: Generator> = Artifact<G, G::Yield, G::Return>;

/// Holds the state of the build output of a node in the compiled DAG.
///
/// Typically follows `Empty -> Waiting -> Some(_) -> Emptied`.
pub(super) enum OutputState<G: Generator> {
    Emptied,
    Empty,
    Waiting,
    Some(GenArtifact<G>),
}

impl<G: Generator> Default for OutputState<G> {
    fn default() -> Self {
        Self::Empty
    }
}

impl<G: Generator> OutputState<G> {
    pub(super) fn move_output(&mut self) -> Option<GenArtifact<G>> {
        std::mem::replace(self, Self::Emptied).into_output()
    }

    pub(super) fn set_output(&mut self, output: GenArtifact<G>) -> Option<GenArtifact<G>> {
        std::mem::replace(self, Self::Some(output)).into_output()
    }

    pub(super) fn into_output(self) -> Option<GenArtifact<G>> {
        if let Self::Some(artifact) = self {
            Some(artifact)
        } else {
            None
        }
    }

    pub(super) fn waiting(&mut self) -> Self {
        std::mem::replace(self, Self::Waiting)
    }

    pub(super) fn is_built(&self) -> bool {
        matches!(self, Self::Emptied | Self::Some(_))
    }
}

pub struct CompilerState<'a, G: Generator> {
    pub src: Source<'a>,
    output: OutputState<G>,
    scope: StructuredState<'a, G>,
    refs: BTreeSet<Address>,
}

impl<'a, G: Generator> std::iter::Extend<(String, CompilerState<'a, G>)> for CompilerState<'a, G> {
    #[inline]
    fn extend<T: IntoIterator<Item = (String, CompilerState<'a, G>)>>(&mut self, iter: T) {
        self.scope.extend(iter)
    }
}

impl<'a, G: Generator> CompilerState<'a, G> {
    #[inline]
    pub(super) fn new(source: Source<'a>) -> Self {
        Self {
            src: source,
            output: OutputState::default(),
            scope: StructuredState::default(),
            refs: BTreeSet::default(),
        }
    }

    #[inline]
    pub(super) fn schema(content: &'a Content) -> Self {
        Self::new(Source::Collection(content))
    }

    #[inline]
    pub fn namespace(content: &'a Content) -> Self {
        Self::new(Source::Namespace(content))
    }

    #[inline]
    pub(super) fn get(&self, field: &str) -> Option<&Self> {
        self.scope.get(field)
    }

    #[inline]
    pub(super) fn get_mut(&mut self, field: &str) -> Option<&mut Self> {
        self.scope.get_mut(field)
    }

    #[inline]
    pub(super) fn output(&self) -> &OutputState<G> {
        &self.output
    }

    #[inline]
    pub(super) fn output_mut(&mut self) -> &mut OutputState<G> {
        &mut self.output
    }

    #[inline]
    pub(super) fn scope(&self) -> &StructuredState<'a, G> {
        &self.scope
    }

    #[inline]
    pub(super) fn scope_mut(&mut self) -> &mut StructuredState<'a, G> {
        &mut self.scope
    }

    #[inline]
    pub(super) fn refs(&self) -> &BTreeSet<Address> {
        &self.refs
    }

    #[inline]
    pub(super) fn refs_mut(&mut self) -> &mut BTreeSet<Address> {
        &mut self.refs
    }

    #[inline]
    pub fn entry<'t>(&'t mut self, field: &str) -> Entry<'t, 'a, G> {
        Entry {
            node: self,
            field: field.to_string(),
        }
    }

    #[inline]
    pub fn source(&self) -> &Source<'a> {
        &self.src
    }

    #[inline]
    pub(super) fn move_output(&mut self) -> Option<GenArtifact<G>> {
        self.output.move_output()
    }

    pub(super) fn project(&self, mut to: Address) -> Result<&Self> {
        let mut projected = self;
        while let Some(next) = to.deeper() {
            projected = projected.get(&next).ok_or_else(
                || failed!(target: Release, Compilation => "unknown field: {}", next),
            )?;
        }
        Ok(projected)
    }

    pub(super) fn project_mut(&mut self, mut to: Address) -> Result<&mut Self> {
        let mut projected = self;
        while let Some(next) = to.deeper() {
            projected = projected.get_mut(&next).ok_or_else(
                || failed!(target: Release, Compilation => "unknown field: {}", next),
            )?;
        }
        Ok(projected)
    }
}

pub struct Entry<'t, 'a, G: Generator> {
    pub(super) node: &'t mut CompilerState<'a, G>,
    pub(super) field: String,
}

impl<'t, 'a, G: Generator> Entry<'t, 'a, G> {
    pub fn or_init(self, content: &'a Content) -> &'t mut CompilerState<'a, G> {
        if !self.node.scope.children.contains_key(&self.field) {
            self.node
                .scope
                .insert(self.field.clone(), CompilerState::schema(content));
        }
        self.node.get_mut(&self.field).unwrap()
    }
}

/// A struct to store the views issued to nodes referring to a given recorder.
pub(super) struct ReferenceFactory<G: Generator> {
    issued: BTreeSet<Address>,
    src: Option<SliceRef<G::Yield, G::Return>>,
}

impl<G: Generator> Default for ReferenceFactory<G> {
    fn default() -> Self {
        Self {
            issued: BTreeSet::new(),
            src: None,
        }
    }
}

impl<G> ReferenceFactory<G>
where
    G: Generator,
    GeneratorOutput<G>: Clone,
{
    fn declare(&mut self, from: Address) -> bool {
        self.issued.insert(from)
    }

    fn issue(&mut self, from: &Address) -> Result<TapeView<G::Yield, G::Return>> {
        if !self.issued.contains(from) {
            Err(anyhow!(
                "cannot issue a reference to `{}` unless it was previously declared",
                from
            ))
        } else if let Some(slice_ref) = self.src.as_ref() {
            Ok(slice_ref.new_view())
        } else {
            Err(anyhow!(
                "tried to issue a reference to `{}` before it was built",
                from
            ))
        }
    }

    fn is_ready(&self) -> bool {
        self.src.is_some()
    }

    fn set_source(
        &mut self,
        inner: SliceRef<G::Yield, G::Return>,
    ) -> Option<SliceRef<G::Yield, G::Return>> {
        std::mem::replace(&mut self.src, Some(inner))
    }

    pub(super) fn get_source(&self) -> Option<SliceRef<G::Yield, G::Return>> {
        self.src.as_ref().cloned()
    }
}

/// A table of addresses, relative to a local root of nodes targetted by a reference.
///
/// At each such address, we keep a [`ReferenceFactory`](ReferenceFactory).
pub(super) struct LocalTable<G: Generator> {
    locals: BTreeMap<Address, ReferenceFactory<G>>,
}

impl<G> Default for LocalTable<G>
where
    G: Generator,
{
    fn default() -> Self {
        Self {
            locals: BTreeMap::new(),
        }
    }
}

impl<G> LocalTable<G>
where
    G: Generator,
    GeneratorOutput<G>: Clone,
{
    fn get(&self, to: &Address) -> Option<&ReferenceFactory<G>> {
        self.locals.get(to)
    }

    pub(super) fn values(&self) -> impl Iterator<Item = &ReferenceFactory<G>> {
        self.locals.values()
    }

    fn get_mut(&mut self, to: &Address) -> Option<&mut ReferenceFactory<G>> {
        self.locals.get_mut(to)
    }

    fn declare(&mut self, from: Address, to: Address) -> Result<bool> {
        let local = self.locals.entry(to).or_default();
        Ok(local.declare(from))
    }

    fn issue(&mut self, from: &Address, to: &Address) -> Result<TapeView<G::Yield, G::Return>> {
        self.get_mut(from)
            .ok_or_else(|| anyhow!("no local table entry for `{}`", from))
            .and_then(|factory| factory.issue(to))
    }
}

/// The global table of addresses of nodes that are closest common ancestors of pairs of nodes
/// referring to one another.
///
/// At each such node's address, we keep a [`LocalTable`](LocalTable).
pub(super) struct Symbols<G: Generator = crate::graph::Graph> {
    flattened: BTreeSet<Address>,
    storage: BTreeMap<Address, LocalTable<G>>,
}

impl<G> Symbols<G>
where
    G: Generator,
    GeneratorOutput<G>: Clone,
{
    pub(super) fn new() -> Self {
        Self {
            flattened: BTreeSet::new(),
            storage: BTreeMap::new(),
        }
    }

    pub(super) fn get(&self, at: &Address) -> Option<&LocalTable<G>> {
        self.storage.get(at)
    }

    pub(super) fn get_mut(&mut self, at: &Address) -> Option<&mut LocalTable<G>> {
        self.storage.get_mut(at)
    }

    pub(super) fn paths(&self, to: &Address) -> Vec<(Address, Address)> {
        self.storage
            .iter()
            .filter_map(|(root, table)| {
                let rem = to.as_in(root)?;
                table.locals.get(&rem)?;
                Some((root.clone(), rem))
            })
            .collect()
    }

    pub(super) fn is_built(&self, from: &Address, to: &Address) -> Option<bool> {
        let (root, relative_to) = to.relativize(from);
        Some(self.get(&root)?.get(&relative_to)?.is_ready())
    }

    pub(super) fn targetted(&self, address: &Address) -> bool {
        self.flattened.contains(address)
    }

    pub(super) fn contains(&self, address: &Address) -> bool {
        self.storage.contains_key(address)
    }

    pub(super) fn declare(&mut self, from: Address, to: Address) -> Result<bool> {
        let (common_root, relative_to) = to.relativize(&from);
        let relative_from = from.as_in(&common_root).unwrap();

        if relative_to.is_root() {
            // the reference is a cycle: `to` is referring to itself
            return Err(failed!(
                target: Release,
                "cycle detected: {} is a direct ancestor to {}",
                to,
                from
            ));
        }

        let already_declared = self
            .storage
            .entry(common_root)
            .or_default()
            .declare(relative_from, relative_to)?;

        //self.flattened.insert(common_root.concat(&relative_to.root()));

        self.flattened.insert(to);

        Ok(already_declared)
    }

    pub(super) fn issue(
        &mut self,
        from: &Address,
        to: &Address,
    ) -> Result<TapeView<G::Yield, G::Return>> {
        let (root, relative_what) = to.relativize(from);
        let relative_to = from.as_in(&root).unwrap();
        self.get_mut(&root)
            .ok_or_else(|| anyhow!("no vtable entry for {}", root))
            .and_then(|local_table| local_table.issue(&relative_what, &relative_to))
    }

    pub(super) fn set_source(
        &mut self,
        root: &Address,
        relative: &Address,
        source: SliceRef<G::Yield, G::Return>,
    ) -> Result<()> {
        if self
            .get_mut(root)
            .ok_or_else(|| anyhow!("no vtable entry for {}", root))?
            .get_mut(relative)
            .ok_or_else(|| anyhow!("no local table entry for {}", relative))?
            .set_source(source)
            .map(|_| ())
            .is_some()
        {
            Err(anyhow!("source already set"))
        } else {
            Ok(())
        }
    }
}
