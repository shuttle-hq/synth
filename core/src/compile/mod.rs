use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::iter::{DoubleEndedIterator, FromIterator, IntoIterator};

use anyhow::{Context, Result};
use colored::Colorize;
use synth_gen::prelude::*;

mod utils;
pub use utils::{Cursored, Driver, GeneratorOutput, Scoped, View};

use crate::graph::{Graph, Unwrapped};
use crate::schema::{Content, FieldRef, Namespace};

macro_rules! says {
    ($level:ident, $stage:expr, $color:ident, $content:expr$(, $arg:expr)*$(,)?) => {
	$level!(target: "compiler", concat!("{} ", $content), $stage.bold().$color()$(, $arg)*)
    };
}

macro_rules! stage_1 {
    ($content:expr$(, $arg:expr)*$(,)?) => {
	says!(info, "stage 1", cyan, $content$(,$arg)*)
    };
}

macro_rules! stage_2 {
    ($content:expr$(, $arg:expr)*$(,)?) => {
	says!(info, "stage 2", magenta, $content$(,$arg)*)
    };
}

pub trait Compiler<'a> {
    /// @brokad: API contract: do not inspect `Graph`
    fn build(&mut self, field: &str, content: &'a Content) -> Result<Graph>;

    /// @brokad: API contract: do not inspect `Graph`
    fn get<S: Into<Scope>>(&mut self, field: S) -> Result<Graph>;
}

pub trait Compile {
    fn compile<'a, C: Compiler<'a>>(&'a self, compiler: C) -> Result<Graph>;
}

pub struct StructuredState<'a> {
    children: BTreeMap<String, CompilerState<'a>>,
    ordering: Vec<String>,
}

#[allow(dead_code)]
impl<'a> StructuredState<'a> {
    #[inline]
    fn get(&self, field: &str) -> Option<&CompilerState<'a>> {
        self.children.get(field)
    }

    #[inline]
    fn get_mut(&mut self, field: &str) -> Option<&mut CompilerState<'a>> {
        self.children.get_mut(field)
    }

    fn insert(&mut self, name: String, state: CompilerState<'a>) -> Option<CompilerState<'a>> {
        if self.children.contains_key(&name) {
            let (idx, _) = self
                .ordering
                .iter()
                .enumerate()
                .find(|(_, value)| **value == name)
                .unwrap();
            self.ordering.remove(idx);
        }
        self.ordering.push(name.clone());
        self.children.insert(name, state)
    }
}

impl<'a> std::iter::Extend<(String, CompilerState<'a>)> for StructuredState<'a> {
    fn extend<T: IntoIterator<Item = (String, CompilerState<'a>)>>(&mut self, iter: T) {
        for (name, state) in iter {
            self.insert(name, state);
        }
    }
}

impl<'a> Default for StructuredState<'a> {
    fn default() -> Self {
        Self {
            children: BTreeMap::new(),
            ordering: Vec::new(),
        }
    }
}

pub mod structured_state {
    use super::*;

    pub struct IntoIter<'a> {
        children: BTreeMap<String, CompilerState<'a>>,
        iter: std::vec::IntoIter<String>,
    }

    impl<'a> From<StructuredState<'a>> for IntoIter<'a> {
        #[inline]
        fn from(v: StructuredState<'a>) -> Self {
            Self {
                children: v.children,
                iter: v.ordering.into_iter(),
            }
        }
    }

    impl<'a> Iterator for IntoIter<'a> {
        type Item = (String, CompilerState<'a>);

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            self.iter
                .next()
                .and_then(|name| self.children.remove(&name).map(|state| (name, state)))
        }
    }
}

impl<'a> FromIterator<(String, CompilerState<'a>)> for StructuredState<'a> {
    fn from_iter<I: IntoIterator<Item = (String, CompilerState<'a>)>>(iter: I) -> Self {
        let (ordering, children) = iter
            .into_iter()
            .map(|(name, state)| (name.clone(), (name, state)))
            .unzip();
        Self { children, ordering }
    }
}

impl<'a> IntoIterator for StructuredState<'a> {
    type IntoIter = structured_state::IntoIter<'a>;
    type Item = (String, CompilerState<'a>);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::from(self)
    }
}

pub enum Source<'a> {
    Namespace(&'a Namespace),
    Content(&'a Content),
}

impl<'a> std::fmt::Display for Source<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Namespace(_) => write!(f, "namespace"),
            Self::Content(_) => write!(f, "content"),
        }
    }
}

impl<'a> Source<'a> {
    #[inline]
    #[allow(dead_code)]
    fn as_namespace(&self) -> Result<&'a Namespace> {
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
    #[allow(dead_code)]
    fn as_content(&self) -> Result<&'a Content> {
        match self {
            Self::Content(content) => Ok(content),
            other => Err(failed!(
                target: Release,
                "source node type mismatch: expected content, found {}",
                other
            )),
        }
    }
}

pub enum Artifact<G: Generator> {
    Just(G),
    Driver(Driver<G>),
}

impl Artifact<Graph> {
    fn into_model(self) -> Graph {
        match self {
            Self::Just(g) => g,
            Self::Driver(driver) => Graph::Driver(driver),
        }
    }
}

pub enum OutputState<G: Generator> {
    Emptied,
    Empty,
    Some(Artifact<G>),
}

impl<G: Generator> Default for OutputState<G> {
    fn default() -> Self {
        Self::Empty
    }
}

impl<G: Generator> OutputState<G> {
    fn move_output(&mut self) -> Option<Artifact<G>> {
        std::mem::replace(self, Self::Emptied).into_output()
    }

    fn set_output(&mut self, output: Artifact<G>) -> Option<Artifact<G>> {
        std::mem::replace(self, Self::Some(output)).into_output()
    }

    fn into_output(self) -> Option<Artifact<G>> {
        if let Self::Some(artifact) = self {
            Some(artifact)
        } else {
            None
        }
    }

    fn is_built(&self) -> bool {
        matches!(self, Self::Emptied | Self::Some(_))
    }
}

pub struct CompilerState<'a> {
    src: Source<'a>,
    compiled: OutputState<Graph>,
    scope: StructuredState<'a>,
    refs: BTreeSet<Scope>,
}

impl<'a> std::iter::Extend<(String, CompilerState<'a>)> for CompilerState<'a> {
    #[inline]
    fn extend<T: IntoIterator<Item = (String, CompilerState<'a>)>>(&mut self, iter: T) {
        self.scope.extend(iter)
    }
}

#[derive(Default, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Scope(VecDeque<String>);

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut chunks = self.iter();
        write!(f, "{}", chunks.next().unwrap_or("{top-level}"))?;
        for chunk in chunks {
            write!(f, ".{}", chunk)?;
        }
        Ok(())
    }
}

#[allow(dead_code)]
impl Scope {
    #[inline]
    fn new_root() -> Self {
        Self(VecDeque::new())
    }

    #[inline]
    fn is_root(&self) -> bool {
        self.0.is_empty()
    }

    fn root(&self) -> Self {
        self.iter().map(|node| node.to_string()).take(1).collect()
    }

    #[inline]
    fn iter(&self) -> impl DoubleEndedIterator<Item = &str> {
        self.0.iter().map(|value| value.as_str())
    }

    #[inline]
    fn deeper(&mut self) -> Option<String> {
        self.0.pop_front()
    }

    #[inline]
    fn shallower(&mut self) -> Option<String> {
        self.0.pop_back()
    }

    #[inline]
    fn within(&mut self, scope: &str) -> &mut Self {
        self.0.push_front(scope.to_string());
        self
    }

    #[inline]
    fn as_within(mut self, scope: &str) -> Self {
        self.within(scope);
        self
    }

    #[inline]
    fn at(&mut self, attribute: &str) -> &mut Self {
        self.0.push_back(attribute.to_string());
        self
    }

    #[inline]
    fn as_at(mut self, attribute: &str) -> Self {
        self.at(attribute);
        self
    }

    fn as_local_to(&self, root: &Self) -> Option<Self> {
        if root.0.len() > self.0.len() {
            None
        } else {
            Some(
                self.iter()
                    .zip(root.iter())
                    .skip_while(|(left, right)| left == right)
                    .map(|(left, _)| left.to_string())
                    .collect(),
            )
        }
    }

    pub fn as_in(&self, other: &Self) -> Option<Self> {
        let mut out = self.clone();
        for level in other.iter() {
            if *level != out.deeper()? {
                return None;
            }
        }
        Some(out)
    }

    fn concat(&self, other: &Self) -> Self {
        let mut out = self.clone();
        out.extend(other.clone().into_iter());
        out
    }

    fn relativize(&self, with: &Self) -> (Self, Self) {
        let root = self.common_root(with);
        let relative = self.as_in(&root).unwrap();
        (root, relative)
    }

    fn common_root(&self, other: &Self) -> Self {
        self.iter()
            .zip(other.iter())
            .take_while(|(left, right)| left == right)
            .map(|(left, _)| left.to_string())
            .collect()
    }
}

impl std::iter::FromIterator<String> for Scope {
    #[inline]
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl std::iter::Extend<String> for Scope {
    #[inline]
    fn extend<T: IntoIterator<Item = String>>(&mut self, iter: T) {
        self.0.extend(iter)
    }
}

impl std::iter::IntoIterator for Scope {
    type Item = String;

    type IntoIter = std::collections::vec_deque::IntoIter<String>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<FieldRef> for Scope {
    #[inline]
    fn from(field: FieldRef) -> Self {
        field.into_iter().collect()
    }
}

#[allow(dead_code)]
impl<'a> CompilerState<'a> {
    #[inline]
    fn new(source: Source<'a>) -> Self {
        Self {
            src: source,
            compiled: OutputState::default(),
            scope: StructuredState::default(),
            refs: BTreeSet::default(),
        }
    }

    #[inline]
    fn content(content: &'a Content) -> Self {
        Self::new(Source::Content(content))
    }

    #[inline]
    fn namespace(namespace: &'a Namespace) -> Self {
        Self::new(Source::Namespace(namespace))
    }

    #[inline]
    fn get(&self, field: &str) -> Option<&CompilerState<'a>> {
        self.scope.get(field)
    }

    #[inline]
    fn get_mut(&mut self, field: &str) -> Option<&mut CompilerState<'a>> {
        self.scope.get_mut(field)
    }

    #[inline]
    fn entry<'t>(&'t mut self, field: &str) -> Entry<'t, 'a> {
        Entry {
            node: self,
            field: field.to_string(),
        }
    }

    #[inline]
    fn source(&self) -> &Source<'a> {
        &self.src
    }

    #[inline]
    fn move_output(&mut self) -> Option<Artifact<Graph>> {
        self.compiled.move_output()
    }

    fn project(&mut self, mut to: Scope) -> Result<&mut Self> {
        let mut projected = self;
        while let Some(next) = to.deeper() {
            projected = projected.get_mut(&next).ok_or_else(
                || failed!(target: Release, Compilation => "unknown field: {}", next),
            )?;
        }
        Ok(projected)
    }
}

pub struct Entry<'t, 'a> {
    node: &'t mut CompilerState<'a>,
    field: String,
}

impl<'t, 'a> Entry<'t, 'a> {
    #[allow(dead_code)]
    fn or_build_within<F>(self, f: F) -> Result<&'t mut CompilerState<'a>>
    where
        for<'tt> F: FnOnce(&'tt mut CompilerState<'a>) -> ContentCompiler<'tt, 'a>,
    {
        if !self.node.compiled.is_built() {
            f(self.node).compile()?;
        }
        if let Some(node) = self.node.get_mut(&self.field) {
            return Ok(node);
        }
        Err(failed!(
            target: Release,
            "could not find field: {}",
            self.field
        ))
    }

    fn or_init(self, content: &'a Content) -> &'t mut CompilerState<'a> {
        if !self.node.scope.children.contains_key(&self.field) {
            self.node
                .scope
                .insert(self.field.clone(), CompilerState::content(content));
        }
        self.node.get_mut(&self.field).unwrap()
    }
}

pub struct NamespaceCompiler<'a> {
    state: CompilerState<'a>,
    vtable: Symbols,
}

impl<'a> NamespaceCompiler<'a> {
    pub fn new_at(state: CompilerState<'a>) -> Self {
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
        stage_1!("discovery");

        let crawler = Crawler {
            cursor: &mut self.state,
            table: &mut self.vtable,
            position: Scope::new_root(),
        };

        crawler.compile()?;

        stage_2!("building");

        let mut visits = VecDeque::<Scope>::new();

        visits.push_back(Scope::new_root());

        while let Some(next_scope) = visits.pop_back() {
            stage_2!("{} nodes left", visits.len());
            stage_2!("{}: entering scope", next_scope);

            let next = self.state.project(next_scope.clone())?;
            let scope = &mut next.scope;
            let compiled = &mut next.compiled;
            let refs = &mut next.refs;

            if compiled.is_built() {
                stage_2!("{}: already built", next_scope);
                continue;
            }

            let own_dependencies: Vec<Scope> = scope
                .ordering
                .iter()
                .filter(|attr| !scope.children.get(*attr).unwrap().compiled.is_built())
                .map(|attr| next_scope.clone().as_at(attr))
                .collect();
            stage_2!(
                "{}: {} dependencies left",
                next_scope,
                own_dependencies.len()
            );

            let mut foreign_dependencies = Vec::new();
            for reference in refs.iter() {
                if !self.vtable.is_built(reference, &next_scope)? {
                    foreign_dependencies.push(reference.clone());
                }
            }
            stage_2!(
                "{}: {} references left",
                next_scope,
                foreign_dependencies.len()
            );

            if !own_dependencies.is_empty() || !foreign_dependencies.is_empty() {
                visits.push_back(next_scope);
                visits.extend(own_dependencies.into_iter().chain(foreign_dependencies));
                continue;
            }

            let mut drivers = BTreeMap::new();

            let ctx = self
                .vtable
                .get(&next_scope)
                .map(|locals| {
                    let keys: Vec<_> = locals.locals.keys().map(|key| key.to_string()).collect();
                    keys.as_slice().join(", ")
                })
                .unwrap_or_else(|_| "{empty}".to_string());
            stage_2!("{}: context extends to {}", next_scope, ctx);

            let content_compiler = ContentCompiler {
                scope: next_scope.clone(),
                cursor: next,
                drivers: &mut drivers,
                vtable: &mut self.vtable,
            };

            stage_2!("{}: building", next_scope);
            let mut model = content_compiler
                .compile()
                .with_context(|| format!("at `{}`", &next_scope))?;
            stage_2!("{}: done", next_scope);

            if let Ok(locals) = self.vtable.get(&next_scope) {
                let cursors = locals.extract();
                let mut order: Vec<_> = drivers.keys().cloned().collect();
                order.sort_by(|left, right| {
                    if locals.closure(left).contains(right) {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Greater
                    }
                });
                model = Graph::Scoped(Scoped {
                    cursors,
                    drivers,
                    order,
                    src: Box::new(model),
                    is_complete: true,
                });
            }

            let artifact = if self.vtable.flattened.contains(&next_scope) {
                stage_2!("{}: targetted", next_scope);
                let (driver, mut cursor) = utils::channel(model);
                for (root, tail) in self.vtable.paths(&next_scope) {
                    // @brokad: keep this order for future proofing
                    // when those become trees
                    let next = cursor;
                    cursor = next.subset();
                    self.vtable.set(&root, &tail, next)?;
                    stage_2!(
                        "{}: shared node at {} to relative {} ready",
                        next_scope,
                        root,
                        tail
                    );
                }
                Artifact::Driver(driver)
            } else {
                Artifact::Just(model)
            };

            next.compiled.set_output(artifact);
        }

        Ok(self.state.move_output().unwrap().into_model())
    }
}

pub struct ContentCompiler<'c, 'a: 'c> {
    scope: Scope,
    cursor: &'c mut CompilerState<'a>,
    drivers: &'c mut BTreeMap<Scope, Driver<Graph>>,
    vtable: &'c mut Symbols,
}

impl<'c, 'a: 'c> ContentCompiler<'c, 'a> {
    fn compile(self) -> Result<Graph> {
        match self.cursor.source() {
            Source::Namespace(namespace) => namespace.compile(self),
            Source::Content(content) => content.compile(self),
        }
    }
}

impl<'c, 'a: 'c> Compiler<'a> for ContentCompiler<'c, 'a> {
    fn build(&mut self, field: &str, _content: &'a Content) -> Result<Graph> {
        stage_2!("{}: moving out of {}", self.scope, field);
        let model = self
            .cursor
            .get_mut(field)
            .ok_or_else(|| failed!(target: Release, Compilation => "undefined: {}", field))?
            .move_output()
            .ok_or_else(
                || failed!(target: Release, Compilation => "dependency not satisfied: {}", field),
            )?
            .into_model();
        let as_scope = Scope::new_root().as_within(field);
        match self
            .vtable
            .get_mut(&self.scope)
            .and_then(|locals| locals.get_mut(&as_scope))
        {
            Ok(factory) => {
                // model is a driver
                match model {
                    Graph::Driver(driver) => self.drivers.insert(as_scope, driver),
                    _ => {
                        return Err(
                            failed!(target: Release, Compilation => "where has the driver gone?"),
                        )
                    }
                };
                let as_view = factory.issue(&Scope::new_root())?.ok_or_else(
                    || failed!(target: Release, Compilation => "reference not built: {}", field),
                )?;
                Ok(Graph::View(Unwrapped::wrap(as_view)))
            }
            Err(_) => Ok(model),
        }
    }

    fn get<S: Into<Scope>>(&mut self, field: S) -> Result<Graph> {
        let as_scope = field.into();
        let view = self.vtable.issue(&as_scope, &self.scope)?.ok_or_else(
            || failed!(target: Release, Compilation => "reference not built: {}", as_scope),
        )?;
        Ok(Graph::View(Unwrapped::wrap(view)))
    }
}

pub struct ReferenceFactory<G: Generator> {
    issued: BTreeSet<Scope>,
    src: Option<Cursored<G>>,
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
    fn issue(&mut self, to: &Scope) -> Result<Option<View<G>>> {
        if !self.issued.contains(to) {
            Err(failed!(target: Release, Compilation => "undeclared reference to {}", to))
        } else {
            Ok(self.src.as_ref().map(|cursored| cursored.view()))
        }
    }

    fn is_built(&self) -> bool {
        self.src.is_some()
    }

    fn set(&mut self, cursor: Cursored<G>) -> Option<Cursored<G>> {
        std::mem::replace(&mut self.src, Some(cursor))
    }

    fn declare(&mut self, to: Scope) -> bool {
        self.issued.insert(to)
    }
}

pub struct LocalTable<G: Generator> {
    locals: BTreeMap<Scope, ReferenceFactory<G>>,
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
    /// @brokad: fix panic
    fn extract(&self) -> BTreeMap<Scope, Cursored<G>> {
        self.locals
            .iter()
            .map(|(scope, factory)| (scope.clone(), factory.src.as_ref().unwrap().clone()))
            .collect()
    }

    fn closure(&self, what: &Scope) -> BTreeSet<Scope> {
        let what_root = what.root();
        self.locals
            .iter()
            .filter(|(scope, _factory)| scope.root() == what_root)
            .flat_map(|(_scope, factory)| factory.issued.iter().map(|issued| issued.root()))
            .collect()
    }

    fn get(&self, to: &Scope) -> Result<&ReferenceFactory<G>> {
        self.locals
            .get(to)
            .ok_or_else(|| failed!(target: Release, "undefined reference to {}", to))
    }

    fn get_mut(&mut self, to: &Scope) -> Result<&mut ReferenceFactory<G>> {
        self.locals
            .get_mut(to)
            .ok_or_else(|| failed!(target: Release, Compilation => "undeclared: {}", to))
    }

    #[allow(dead_code)]
    fn ensure_no_cycle(&self, what: &Scope, to: &Scope) -> Result<()> {
        let what_cl = self.closure(what);
        let to_cl = self.closure(to);

        let what_root = what.root();
        if what_cl.contains(&what_root) {
            return Err(failed!(
                target: Release,
                "the closure of {} contains {}",
                what,
                what_root
            ));
        }

        let to_root = to.root();
        if to_cl.contains(&to_root) {
            return Err(failed!(
                target: Release,
                "the closure of {} contains {}",
                to,
                to_root
            ));
        }

        if to_cl.intersection(&what_cl).next().is_some() {
            return Err(failed!(
                target: Release,
                "the closures of {} and {} intersect each other",
                what,
                to
            ));
        }

        Ok(())
    }

    fn issue(&mut self, what: &Scope, to: &Scope) -> Result<Option<View<G>>> {
        self.locals
            .get_mut(what)
            .ok_or_else(|| failed!(target: Release, "undefined reference to {}", what))
            .and_then(|factory| factory.issue(to))
    }

    fn declare(&mut self, what: Scope, to: Scope) -> Result<bool> {
        if self.closure(&to).contains(&what.root()) {
            Err(failed!(
                target: Release,
                "cycle detected: cannot add a reference to {} from {}",
                what,
                to
            ))
        } else {
            let top_level = self.locals.entry(what.root()).or_default();
            top_level.declare(Scope::new_root());
            let local = self.locals.entry(what).or_default();
            Ok(local.declare(to))
        }
    }
}

pub struct Symbols<G: Generator = Graph> {
    flattened: BTreeSet<Scope>,
    storage: BTreeMap<Scope, LocalTable<G>>,
}

impl<G> Symbols<G>
where
    G: Generator,
    GeneratorOutput<G>: Clone,
{
    fn new() -> Self {
        Self {
            flattened: BTreeSet::new(),
            storage: BTreeMap::new(),
        }
    }

    fn paths(&self, to: &Scope) -> Vec<(Scope, Scope)> {
        let mut out = self
            .storage
            .iter()
            .filter_map(|(what, table)| {
                let rem = to.as_in(what)?;
                table.locals.get(&rem)?;
                Some((what.clone(), rem))
            })
            .collect::<Vec<_>>();
        out.sort_by_key(|(start, _)| -(start.0.len() as i64));
        out
    }

    fn is_built(&self, what: &Scope, to: &Scope) -> Result<bool> {
        let (root, relative) = what.relativize(to);
        Ok(self.get(&root)?.get(&relative)?.is_built())
    }

    fn declare(&mut self, what: Scope, to: Scope) -> Result<bool> {
        let (root, relative) = what.relativize(&to);

        if relative.is_root() {
            // the reference is a cycle: `to` is referring to itself
            return Err(failed!(
                target: Release,
                "cycle detected: {} is a direct ancestor to {}",
                what,
                to
            ));
        }

        let relative_root = relative.root();
        let relative_to = to.as_in(&root).unwrap();
        let already = self
            .storage
            .entry(root.clone())
            .or_default()
            .declare(relative, relative_to)?;
        self.flattened.insert(root.concat(&relative_root));
        self.flattened.insert(what);
        Ok(already)
    }

    fn get(&self, at: &Scope) -> Result<&LocalTable<G>> {
        self.storage
            .get(at)
            .ok_or_else(|| failed!(target: Release, Compilation => "undeclared: {}", at))
    }

    fn get_mut(&mut self, at: &Scope) -> Result<&mut LocalTable<G>> {
        self.storage
            .get_mut(at)
            .ok_or_else(|| failed!(target: Release, Compilation => "undeclared: {}", at))
    }

    fn set(
        &mut self,
        root: &Scope,
        relative: &Scope,
        cursor: Cursored<G>,
    ) -> Result<Option<Cursored<G>>> {
        Ok(self.get_mut(root)?.get_mut(relative)?.set(cursor))
    }

    fn issue(&mut self, what: &Scope, to: &Scope) -> Result<Option<View<G>>> {
        let (root, relative_what) = what.relativize(to);
        let relative_to = to.as_in(&root).unwrap();
        self.get_mut(&root)
            .with_context(|| anyhow!(
                "looking for the common root to {} and {}",
                what,
                to
            ))
            .and_then(|local_table| local_table.issue(&relative_what, &relative_to))
    }
}

pub struct Crawler<'t, 'a> {
    cursor: &'t mut CompilerState<'a>,
    table: &'t mut Symbols,
    position: Scope,
}

impl<'t, 'a: 't> Crawler<'t, 'a> {
    fn as_at(&mut self, field: &str, content: &'a Content) -> Crawler<'_, 'a> {
        let position = self.position.clone().as_at(field);
        stage_1!("entering {}", position);
        Crawler {
            cursor: self.cursor.entry(field).or_init(content),
            table: self.table,
            position,
        }
    }

    fn compile(self) -> Result<()> {
        match self.cursor.source() {
            Source::Namespace(namespace) => namespace.compile(self)?,
            Source::Content(content) => content.compile(self)?,
        };
        Ok(())
    }
}

impl<'t, 'a: 't> Compiler<'a> for Crawler<'t, 'a> {
    fn build(&mut self, field: &str, content: &'a Content) -> Result<Graph> {
        if let Err(err) = self.as_at(field, content).compile() {
            warn!(target: "compiler", "{} node {} err'ed at visit: {}", "stage 1".bold().cyan(), field, err);
        }
        Ok(Graph::null())
    }

    fn get<S: Into<Scope>>(&mut self, field: S) -> Result<Graph> {
        let as_scope: Scope = field.into();
        stage_1!("linking {} to {}", as_scope, self.position);
        self.table
            .declare(as_scope.clone(), self.position.clone())?;
        self.cursor.refs.insert(as_scope);
        Ok(Graph::null())
    }
}
