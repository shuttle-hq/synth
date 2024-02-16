use synth_gen::prelude::{Generator, GeneratorState};

use std::cell::RefCell;
use std::iter::IntoIterator;
use std::ops::Range;
use std::rc::Rc;

use crate::graph::prelude::Rng;

pub struct Slice {
    generation: usize,
    start: usize,
}

pub struct Tape<Y, R> {
    slices: Vec<Slice>,
    buffer: Vec<GeneratorState<Y, R>>,
}

impl<Y, R> Tape<Y, R> {
    fn new() -> Self {
        Self {
            slices: Vec::new(),
            buffer: Vec::new(),
        }
    }

    fn push_back(&mut self, state: GeneratorState<Y, R>) {
        self.buffer.push(state)
    }

    fn get(&self, idx: usize) -> Option<&GeneratorState<Y, R>> {
        self.buffer.get(idx)
    }

    fn reset(&mut self, idx: usize) {
        let slice = self.slices.get_mut(idx).unwrap();
        slice.generation += 1;
        slice.start = self.buffer.len();
    }

    fn get_generation(&self, idx: usize) -> usize {
        self.slices.get(idx).unwrap().generation
    }

    fn new_range(&self, idx: usize) -> Range<usize> {
        self.slices.get(idx).unwrap().start..self.buffer.len()
    }
}

pub type SharedTape<Y, R> = Rc<RefCell<Tape<Y, R>>>;

pub(super) struct SliceRef<Y, R> {
    index: usize,
    tape: SharedTape<Y, R>,
}

pub(super) type GeneratorSliceRef<G: Generator> = SliceRef<G::Yield, G::Return>;

impl<Y, R> Clone for SliceRef<Y, R> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            tape: self.tape.clone(),
        }
    }
}

impl<Y, R> SliceRef<Y, R> {
    pub(super) fn reset(&self) {
        (*self.tape).borrow_mut().reset(self.index)
    }

    pub(super) fn new_view(&self) -> TapeView<Y, R> {
        TapeView(TapeViewImpl {
            slice: self.clone(),
            generation: 0,
            range: Range::default(),
        })
    }

    fn new_range(&self) -> Range<usize> {
        (*self.tape).borrow().new_range(self.index)
    }

    fn get_generation(&self) -> usize {
        (*self.tape).borrow().get_generation(self.index)
    }
}

pub(super) struct RecorderImpl<G, Y, R> {
    inner: G,
    tape: SharedTape<Y, R>,
}

impl<G, Y, R> RecorderImpl<G, Y, R> {
    fn new_slice(&self) -> SliceRef<Y, R> {
        let index = {
            let mut tape = (*self.tape).borrow_mut();
            let slice = Slice {
                generation: 0,
                start: tape.buffer.len(),
            };
            tape.slices.push(slice);
            tape.slices.len() - 1
        };
        SliceRef {
            index,
            tape: self.tape.clone(),
        }
    }
}

impl<G, Y, R> Generator for RecorderImpl<G, Y, R>
where
    G: Generator<Yield = Y, Return = R>,
    GeneratorState<Y, R>: Clone,
{
    type Yield = Y;

    type Return = R;

    fn next<RR: Rng>(&mut self, rng: &mut RR) -> GeneratorState<Self::Yield, Self::Return> {
        let state = self.inner.next(rng);
        (*self.tape).borrow_mut().push_back(state.clone());
        state
    }
}

impl<G, Y, R> RecorderImpl<G, Y, R>
where
    G: Generator<Yield = Y, Return = R>,
{
    fn complete<RR: Rng>(&mut self, rng: &mut RR) {
        #[allow(clippy::blocks_in_conditions)]
        while {
            let state = self.inner.next(rng);
            let is_complete = state.is_complete();
            (*self.tape).borrow_mut().push_back(state);
            !is_complete
        } {}
    }
}

/// A generator that drives an inner generator and clones the result into a buffer before
/// passing them through.
///
/// Closely related to [`TapeView`](TapeView).
pub struct Recorder<G, Y, R>(pub(super) RecorderImpl<G, Y, R>);

pub type GeneratorRecorder<G: Generator> = Recorder<G, G::Yield, G::Return>;

impl<G, Y, R> Recorder<G, Y, R> {
    pub(super) fn new_slice(&self) -> SliceRef<Y, R> {
        self.0.new_slice()
    }

    pub(super) fn wrap(inner: G) -> Self {
        Self(RecorderImpl {
            inner,
            tape: Rc::new(RefCell::new(Tape::new())),
        })
    }
}

impl<G, Y, R> Recorder<G, Y, R>
where
    G: Generator<Yield = Y, Return = R>,
{
    fn complete<RR: Rng>(&mut self, rng: &mut RR) {
        self.0.complete(rng)
    }
}

pub(super) struct TapeViewImpl<Y, R> {
    slice: SliceRef<Y, R>,
    generation: usize,
    range: Range<usize>,
}

impl<Y, R> TapeViewImpl<Y, R> {
    fn is_obsolete(&self) -> bool {
        self.slice.get_generation() != self.generation
    }

    fn reset(&mut self) -> bool {
        self.generation = self.slice.get_generation();
        self.range = self.slice.new_range();
        !self.range.is_empty()
    }
}

impl<Y, R> Generator for TapeViewImpl<Y, R>
where
    GeneratorState<Y, R>: Clone,
{
    type Yield = Y;
    type Return = Option<R>;

    fn next<RR: Rng>(&mut self, rng: &mut RR) -> GeneratorState<Self::Yield, Self::Return> {
        if self.is_obsolete() && !self.reset() {
            return GeneratorState::Complete(None);
        }

        if let Some(idx) = self.range.next() {
            (*self.slice.tape)
                .borrow()
                .get(idx)
                .expect("a view's index was out of bound")
                .clone()
                .map_complete(Some)
        } else if !self.reset() {
            GeneratorState::Complete(None)
        } else {
            self.next(rng)
        }
    }
}

/// A generator that reads its output from a slice into the buffer managed by a
/// [`Recorder`](Recorder).
pub struct TapeView<Y, R>(pub(super) TapeViewImpl<Y, R>);

pub(super) struct OrderedImpl<G, Y, R> {
    is_complete: bool,
    scope: Vec<SliceRef<Y, R>>,
    children: Vec<(String, Recorder<G, Y, R>)>,
    src: G,
}

impl<G, Y, R> Generator for OrderedImpl<G, Y, R>
where
    G: Generator<Yield = Y, Return = R>,
{
    type Yield = Y;
    type Return = R;

    fn next<RR: Rng>(&mut self, rng: &mut RR) -> GeneratorState<Self::Yield, Self::Return> {
        if !self.is_complete {
            self.scope.iter_mut().for_each(|slice| slice.reset());
            self.children.iter_mut().for_each(|(_, recorder)| {
                recorder.complete(rng);
            });
            self.is_complete = true;
        }
        let state = self.src.next(rng);
        self.is_complete = !state.is_complete();
        state
    }
}

/// A generator charged with setting the order of generation of a collection of inner
/// generators.
///
/// This is essential to handle relations correctly as targets of references must always generate
/// first.
///
/// `Ordered` is constructed by compiler after ordering children nodes according to their relative
/// dependencies to one another.
pub struct Ordered<G, Y, R>(pub(super) OrderedImpl<G, Y, R>);

impl<G, Y, R> Ordered<G, Y, R> {
    pub(super) fn new<S, C>(scope: S, children: C, src: G) -> Self
    where
        S: IntoIterator<Item = SliceRef<Y, R>>,
        C: IntoIterator<Item = (String, Recorder<G, Y, R>)>,
    {
        Self(OrderedImpl {
            is_complete: false,
            scope: scope.into_iter().collect(),
            children: children.into_iter().collect(),
            src,
        })
    }
}

/// A generator compositor to establish relations ('links') between nodes of a generator tree.
pub enum Link<G, Y, R> {
    /// A recorder node.
    Recorder(Recorder<G, Y, R>),
    /// A view node.
    View(TapeView<Y, R>),
    /// An ordered node.
    Ordered(Ordered<G, Y, R>),
    /// A variant used in compilation as a placeholder prior to setting the node to its final
    /// value.
    Dummy,
}

impl<G, Y, R> Generator for Link<G, Y, R>
where
    G: Generator<Yield = Y, Return = R>,
    GeneratorState<Y, R>: Clone,
{
    type Yield = Y;

    type Return = Option<R>;

    fn next<RR: Rng>(&mut self, rng: &mut RR) -> GeneratorState<Self::Yield, Self::Return> {
        match self {
            Self::Recorder(Recorder(recorder)) => recorder.next(rng).map_complete(Some),
            Self::View(TapeView(view)) => view.next(rng),
            Self::Ordered(Ordered(ordered)) => ordered.next(rng).map_complete(Some),
            Self::Dummy => panic!("tried to generate values from a dummy"),
        }
    }
}

impl<G, Y, R> Link<G, Y, R> {
    pub fn new_dummy() -> Self {
        Self::Dummy
    }

    pub fn iter_order(&self) -> Option<impl Iterator<Item = &str>> {
        match self {
            Self::Ordered(Ordered(OrderedImpl { children, .. })) => {
                Some(children.iter().map(|(name, _)| name.as_str()))
            }
            _ => None,
        }
    }
}

/// A trait for types that can own an inner [`Link`](Link).
pub trait FromLink: Sized {
    /// The type yielded by the [`Link`](Link).
    type Yield;

    /// The type returned by the [`Link`](Link).
    type Return;

    /// Build a `Self` from an instance of [`Link`](Link).
    fn from_link(link: Link<Self, Self::Yield, Self::Return>) -> Self;

    fn dummy() -> Self {
        Self::from_link(Link::new_dummy())
    }
}
