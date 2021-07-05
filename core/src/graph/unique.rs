use crate::graph::prelude::{Generator, GeneratorState, Rng};
use crate::Graph;
use bloomfilter::Bloom;
use std::collections::HashSet;
use std::hash::Hash;

//pub struct UniqueNode(Unique<Graph>);

pub enum UniqueNode {
    Hash(Unique<Graph>),
    Bloom(UniqueBloom<Graph>),
}

impl Generator for UniqueNode {
    type Yield = <Graph as Generator>::Yield;
    type Return = <Graph as Generator>::Return;

    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        match self {
            UniqueNode::Hash(unique) => unique.next(rng),
            UniqueNode::Bloom(unique_bloom) => unique_bloom.next(rng),
        }
    }
}
/// This struct is a Generator which guarantees yielding unique values.
/// This Generator grows linearly with the amount of values yielded from it's child.
/// For large generation batches it is recommended to use the [`UniqueBloom`] Generator.
pub struct Unique<G>
    where
        G: Generator,
        G::Yield: PartialEq + Hash,
{
    inner: Box<G>,
    seen: HashSet<G::Yield>,
}

impl<G> Unique<G>
    where
        G: Generator,
        G::Yield: PartialEq + Hash,
{
    pub fn new(inner: G) -> Self {
        Self {
            inner: Box::new(inner),
            seen: Default::default(),
        }
    }
}

impl<G> Generator for Unique<G>
    where
        G: Generator,
        G::Yield: PartialEq + Hash + Clone + Eq,
{
    type Yield = G::Yield;
    type Return = G::Return; // probs not right

    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        loop {
            match self.inner.next(rng) {
                GeneratorState::Yielded(yielded) => {
                    // If the set *does not* contain the value it returns true.
                    if !self.seen.contains(&yielded) {
                        self.seen.insert(yielded.clone());
                        return GeneratorState::Yielded(yielded);
                    }
                    continue;
                }
                GeneratorState::Complete(complete) => {
                    return GeneratorState::Complete(complete);
                }
            }
        }
    }
}

/// This struct is a Generator which guarantees yielding unique values.
/// The generator will guarantee unique values, but will not necessarily exhaust it's child's non-unique yields.
/// It is therefore a constant-space probabilistic generator.
pub struct UniqueBloom<G>
    where
        G: Generator,
{
    inner: Box<G>,
    seen: Bloom<G::Yield>,
}

impl<G> Generator for UniqueBloom<G>
    where
        G: Generator,
        G::Yield: Hash,
{
    type Yield = G::Yield;
    type Return = G::Return; // probs not right

    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        loop {
            match self.inner.next(rng) {
                GeneratorState::Yielded(yielded) => {
                    if !self.seen.check_and_set(&yielded) {
                        return GeneratorState::Yielded(yielded);
                    }
                    continue;
                }
                GeneratorState::Complete(complete) => {
                    return GeneratorState::Complete(complete);
                }
            }
        }
    }
}