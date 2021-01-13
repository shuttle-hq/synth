#![feature(try_trait)]
#![forbid(unsafe_code)]
#![allow(clippy::all, type_alias_bounds)]
#![warn(rust_2018_idioms)]
// @brokad: suppressed for now
//#![warn(missing_docs, missing_doc_code_examples)]
#![deny(elided_lifetimes_in_paths)]

//! # bynar
//!
//!

use std::{collections::VecDeque, fmt::Debug, iter::FromIterator, marker::PhantomData, ops::Try};

use rand::{
    distributions::{Distribution, Standard},
    Rng as RandRng,
};

#[cfg(feature = "faker")]
use fake::Dummy as FakerDummy;

/// Keeping just in case, but used nowhere at this point.
#[allow(unused_macros)]
macro_rules! curry_rng {
    ($($id:ident -> $ret:ty,)*) => {
	$(curry_rng!($id -> $ret);)*
    };
    ($id:ident -> $ret:ty) => {
	/// Run the inner generator's `$id` function on the driver's
	/// [`Rng`](crate::Rng).
	pub fn $id(&mut self) -> $ret {
	    self.generator.$id(self.rng)
	}
    };
}

pub mod de;
pub mod ser;

pub mod error;
pub use error::Error;

pub mod value;
pub use value::TokenGeneratorExt;

pub mod read;

/// The standard [`rand::Rng`](rand::Rng) implementation used by this
/// crate.
pub type Rng = rand::rngs::ThreadRng;

pub mod prelude;

#[cfg(feature = "shared")]
pub mod shared;
#[cfg(feature = "shared")]
pub use shared::Shared;

/// The return type of a [`Generator`](crate::Generator)'s
/// [`next`](crate::Generator::next) function.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum GeneratorState<Y, R> {
    /// [`Generator`](crate::Generator) has *yielded* a value of type
    /// `Y`.
    Yielded(Y),
    /// [`Generator`](crate::Generator) has *completed* and returned a
    /// value of type `R`.
    Complete(R),
}

impl<Y, R> GeneratorState<Y, R> {
    pub fn is_yielded(&self) -> bool {
        match self {
            Self::Yielded(_) => true,
            _ => false,
        }
    }

    pub fn is_complete(&self) -> bool {
        match self {
            Self::Complete(_) => true,
            _ => false,
        }
    }

    pub fn into_yielded(self) -> Result<Y, Error> {
        if let GeneratorState::Yielded(y) = self {
            Ok(y)
        } else {
            Err(Error::custom("unexpected EOF"))
        }
    }

    pub fn into_complete(self) -> Result<R, Error> {
        if let GeneratorState::Complete(r) = self {
            Ok(r)
        } else {
            Err(Error::custom("unexpected EOF"))
        }
    }
}

/// A type that cannot be publicly constructed.
///
/// Similar in spirit to the unstable [`!`](std::primitive::never)
/// type.
#[derive(PartialEq, Eq)]
pub struct Never(PhantomData<()>);

impl std::fmt::Debug for Never {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!("Never should never be constructed")
    }
}

/// The core trait of this crate.
///
/// [`Generator`](crate::Generator)s are stateful streams that pull
/// randomness from an [`Rng`](crate::Rng) to yield a sequence of
/// values of type [`Yield`](crate::Generator::Yield). On
/// completion of the stream, they return a value of type
/// [`Return`](crate::Generator::Return).
///
/// Note that [`Generator`](crate::Generator)s are not required to
/// complete in finite time. When they do complete however, they are
/// required to restart anew.
pub trait Generator {
    /// The type this generator yields.
    type Yield;

    /// The type this generator returns on completion.
    type Return;

    /// Step through one item in the stream.
    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return>;

    fn complete(&mut self, rng: &mut Rng) -> Self::Return {
        loop {
            if let GeneratorState::Complete(ret) = self.next(rng) {
                return ret;
            }
        }
    }
}

/// A trait extension for [`Generator`](crate::Generator)s that yield
/// and return values of the same type `D`.
pub trait DiagonalGeneratorExt<D>: Generator<Yield = D, Return = D> + Sized {}

impl<D, T> DiagonalGeneratorExt<D> for T where T: Generator<Yield = D, Return = D> {}

/// A trait extension for [`Generator`](crate::Generator)s that allow
/// for composing complex streams from simple seeds.
pub trait GeneratorExt: Generator + Sized {
    /// Transform every value yielded by the stream into a returned value.
    fn once(self) -> Once<Self> {
        Once(self, None)
    }

    /// Apply a closure to the value returned by a stream.
    fn map<O, F: Fn(Self::Return) -> O>(self, closure: F) -> Map<Self, F, O> {
        Map {
            inner: self,
            closure,
            _output: PhantomData,
        }
    }

    /// The monadic bind operation for [`Generator`](crate::Generator)s.
    ///
    /// Given a closure that constructs a second stream from the value
    /// returned by a first, `and_then` runs the first stream to
    /// completion and then runs the second.
    fn and_then<O, F>(self, closure: F) -> AndThen<Self, F, O>
    where
        F: Fn(Self::Return) -> O,
        O: Generator<Yield = Self::Yield>,
    {
        AndThen {
            inner: self,
            closure,
            output: None,
        }
    }

    /// Semantically the same as [`map`](crate::GeneratorExt::map) but
    /// applied to values yielded by the stream (instead of values
    /// returned by it).
    fn intercept<O, F>(self, closure: F) -> Intercept<Self, F, O>
    where
        F: Fn(Self::Yield) -> O,
    {
        Intercept {
            inner: self,
            closure,
            _output: PhantomData,
        }
    }

    /// Concatenate `self` with `right`.
    ///
    /// First `self` is exhausted to completion, then `right` is
    /// exhausted to completion. The new
    /// [`Generator`](crate::Generator) returns a pair of the values
    /// returned by `self` and `right`.
    fn concatenate<R: Generator>(self, right: R) -> Concatenate<Self, R> {
        Concatenate {
            left: self,
            left_return: None,
            right,
        }
    }

    fn exhaust(self) -> Exhaust<Self> {
        Exhaust { inner: self }
    }

    fn maybe(self) -> Maybe<Self> {
        Maybe {
            inner: self,
            include: false,
        }
    }

    /// Prefix the stream with another.
    #[inline]
    fn prefix<BG>(self, prefix: BG) -> Prefix<BG, Self>
    where
        BG: Generator<Yield = Self::Yield>,
    {
        self.brace(prefix, Empty::new())
    }

    /// Suffix the stream with another.
    #[inline]
    fn suffix<EG>(self, suffix: EG) -> Suffix<Self, EG>
    where
        EG: Generator<Yield = Self::Yield>,
    {
        self.brace(Empty::new(), suffix)
    }

    /// Brace the stream with two others.
    fn brace<BG, EG>(self, begin: BG, end: EG) -> Brace<BG, Self, EG>
    where
        BG: Generator<Yield = Self::Yield>,
        EG: Generator<Yield = Self::Yield>,
    {
        Brace {
            begin,
            inner: self,
            end,
            state: BraceState::Begin,
            complete: None,
        }
    }

    /// Run a closure on every value generated by the stream (both
    /// yielded and returned).
    ///
    /// Useful for debugging.
    fn inspect<F>(self, closure: F) -> Inspect<Self, F>
    where
        F: Fn(&GeneratorState<Self::Yield, Self::Return>),
    {
        Inspect {
            inner: self,
            closure,
        }
    }

    /// Make the stream a
    /// [`PeekableGenerator`](crate::PeekableGenerator).
    ///
    /// This uses a simple [`VecDeque`](std::collections::VecDeque)
    /// buffer to store values generated by `self`.
    fn peekable(self) -> Peek<Self> {
        Peek {
            inner: self,
            buffer: VecDeque::new(),
        }
    }

    /// **TODO** Collect the values yielded by `self` and returns them
    /// in a vec.
    fn aggregate(self) -> Aggregate<Self> {
        Aggregate { inner: self }
    }

    /// **TODO** Repeat inner stream `len` times, returning all
    /// intermediate returned values in a vec.
    fn take(self, len: usize) -> Take<Self> {
        Take {
            inner: self,
            len,
            rem: len,
            ret: Vec::new(),
        }
    }

    fn ok<O: Try<Ok = Self::Return>>(self) -> Okayed<Self, O> {
        Okayed(self.map(O::from_ok))
    }

    #[cfg(feature = "shared")]
    fn shared(self) -> Shared<Self> {
        Shared::new(self)
    }

    fn replay(self, len: usize) -> Replay<Self> {
        Replay {
            inner: self,
            len,
            idx: 0,
            rem: Some(len),
            buf: Vec::new(),
            ret: None,
        }
    }

    fn replay_forever(self) -> Replay<Self> {
        Replay {
            inner: self,
            len: 0,
            idx: 0,
            rem: None,
            buf: Vec::new(),
            ret: None,
        }
    }

    /// **TODO** Add a label to values yielded by `self`.
    fn with_label<L>(self, label: L) -> Labeled<L, Self> {
        Labeled { label, inner: self }
    }

    /// Box the [`Generator`](Generator).
    fn boxed(self) -> Box<dyn Generator<Yield = Self::Yield, Return = Self::Return>>
    where
        Self: 'static,
    {
        Box::new(self)
    }
}

pub struct Okayed<G, O>(Map<G, fn(O::Ok) -> O, O>)
where
    G: Generator,
    O: Try<Ok = G::Return>;

impl<G, O> Generator for Okayed<G, O>
where
    G: Generator,
    O: Try<Ok = G::Return>,
{
    type Yield = G::Yield;

    type Return = O;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        self.0.next(rng)
    }
}

impl<T> GeneratorExt for T where T: Generator {}

/// A primitive random value generator that never terminates.
///
/// This [`Generator`](crate::Generator) yields random values of type
/// `T` generated by a
/// [`Distribution`](rand::distributions::Distribution) `D`.
pub struct Seed<T, D = Standard>(D, PhantomData<T>);

impl<D> Seed<(), D> {
    /// Create a new seed from a distribution `D`.
    pub fn new_with<TT>(dist: D) -> Seed<TT, D>
    where
        D: Distribution<TT>,
    {
        Seed(dist, PhantomData)
    }
}

impl Seed<()> {
    pub fn new<T>() -> Seed<T>
    where
        Standard: Distribution<T>,
    {
        Seed::new_with(Standard)
    }
}

/// Create a seed of random values of `T` with `rand::Distribution` `D`.
pub fn random<T, D: Distribution<T>>(dist: D) -> Seed<T, D> {
    Seed::new_with(dist)
}

impl<D, T> Generator for Seed<T, D>
where
    D: Distribution<T>,
{
    type Yield = T;

    type Return = Never;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        GeneratorState::Yielded(self.0.sample(rng))
    }
}

/// A generator of dummy values.
pub struct Dummy<T, D>(D, PhantomData<T>);

impl<D> Dummy<(), D> {
    /// Create a seed of dummy values of type `T` with [`fake::Dummy`](fake::Dummy) `D`.
    ///
    /// See [fake::faker](fake::faker) for a list of available dummies.
    ///
    /// # Example
    /// ```
    /// # use synth_generator::prelude::*;
    /// # fn main() {
    /// let first_name: String = synth_generator::Dummy::new(faker::name::en::FirstName())
    ///     .once()
    ///     .complete(&mut thread_rng());
    /// println!("{}", first_name)
    /// # }
    #[cfg(feature = "faker")]
    pub fn new<TT>(dummy: D) -> Dummy<TT, D>
    where
        TT: FakerDummy<D>,
    {
        Dummy(dummy, PhantomData)
    }
}

#[cfg(feature = "faker")]
impl<T, D> Generator for Dummy<T, D>
where
    T: FakerDummy<D>,
{
    type Yield = T;

    type Return = Never;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        GeneratorState::Yielded(T::dummy_with_rng(&self.0, rng))
    }
}

/// A generator that takes values yielded by another and transforms
/// them into returned values.
///
/// This `struct` is created by the
/// [`once`](crate::GeneratorExt::once) method on
/// [`Generator`](crate::Generator).
pub struct Once<G: Generator>(G, Option<G::Yield>);

impl<G> Generator for Once<G>
where
    G: Generator,
    G::Yield: Clone,
{
    type Yield = G::Yield;

    type Return = G::Yield;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        if let Some(y) = std::mem::replace(&mut self.1, None) {
            GeneratorState::Complete(y)
        } else {
            match self.0.next(rng) {
                GeneratorState::Yielded(y) => {
                    self.1 = Some(y.clone());
                    GeneratorState::Yielded(y)
                }
                GeneratorState::Complete(_) => self.next(rng),
            }
        }
    }
}

pub trait TryGeneratorExt: Sized
where
    Self: Generator,
    Self::Return: Try,
{
    fn and_then_try<F, O>(self, f: F) -> AndThenTry<Self, F, O>
    where
        F: Fn(<<Self as Generator>::Return as Try>::Ok) -> O,
        O: Generator<Yield = Self::Yield>,
        O::Return: Try<Error = <Self::Return as Try>::Error>,
    {
        AndThenTry {
            inner: self,
            closure: f,
            output: None,
        }
    }
}

impl<G> TryGeneratorExt for G
where
    G: Generator,
    Self::Return: Try,
{
}

pub struct AndThenTry<G, F, O> {
    inner: G,
    closure: F,
    output: Option<O>,
}

impl<G, F, O> Generator for AndThenTry<G, F, O>
where
    G: Generator,
    G::Return: Try,
    F: Fn(<<G as Generator>::Return as Try>::Ok) -> O,
    O: Generator<Yield = G::Yield>,
    O::Return: Try<Error = <G::Return as Try>::Error>,
{
    type Yield = G::Yield;

    type Return = O::Return;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        if let Some(output) = self.output.as_mut() {
            match output.next(rng) {
                GeneratorState::Yielded(y) => GeneratorState::Yielded(y),
                GeneratorState::Complete(r) => {
                    self.output = None;
                    GeneratorState::Complete(r)
                }
            }
        } else {
            match self.inner.next(rng) {
                GeneratorState::Yielded(y) => GeneratorState::Yielded(y),
                GeneratorState::Complete(r) => match r.into_result() {
                    Ok(r_ok) => {
                        self.output = Some((self.closure)(r_ok));
                        self.next(rng)
                    }
                    Err(r_err) => {
                        let complete = <O::Return as Try>::from_error(r_err);
                        GeneratorState::Complete(complete)
                    }
                },
            }
        }
    }
}

/// A generator that applies a closure to the value returned by another.
///
/// This `struct` is constructed by the
/// [`map`](crate::GeneratorExt::map) method on
/// [`Generator`](crate::Generator).
pub struct Map<G, F, O> {
    inner: G,
    closure: F,
    _output: PhantomData<O>,
}

impl<G, F, O> Generator for Map<G, F, O>
where
    G: Generator,
    F: Fn(G::Return) -> O,
{
    type Yield = G::Yield;

    type Return = O;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        match self.inner.next(rng) {
            GeneratorState::Yielded(y) => GeneratorState::Yielded(y),
            GeneratorState::Complete(r) => GeneratorState::Complete((self.closure)(r)),
        }
    }
}

/// A generator that exhausts a first generator before using a closure
/// on the returned value in order to build and exhaust a second
/// generator.
///
/// This `struct` is constructed by the
/// [`and_then`](crate::GeneratorExt::and_then) method on
/// [`Generator`](crate::Generator)
pub struct AndThen<G, F, O> {
    inner: G,
    closure: F,
    output: Option<O>,
}

impl<G, F, O> Generator for AndThen<G, F, O>
where
    G: Generator,
    F: Fn(G::Return) -> O,
    O: Generator<Yield = G::Yield>,
{
    type Yield = G::Yield;

    type Return = O::Return;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        if let Some(output) = self.output.as_mut() {
            match output.next(rng) {
                GeneratorState::Yielded(y) => GeneratorState::Yielded(y),
                GeneratorState::Complete(r) => {
                    self.output = None;
                    GeneratorState::Complete(r)
                }
            }
        } else {
            match self.inner.next(rng) {
                GeneratorState::Yielded(y) => GeneratorState::Yielded(y),
                GeneratorState::Complete(r) => {
                    self.output = Some((self.closure)(r));
                    self.next(rng)
                }
            }
        }
    }
}

/// A generator that applies a closure to values yielded by another.
///
/// This struct is created by the
/// [`intercept`](crate::GeneratorExt::intercept) method on
/// [`Generator`](crate::Generator).
pub struct Intercept<G, F, O> {
    inner: G,
    closure: F,
    _output: PhantomData<O>,
}

impl<G, F, O> Generator for Intercept<G, F, O>
where
    G: Generator,
    F: Fn(G::Yield) -> O,
{
    type Yield = O;

    type Return = G::Return;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        match self.inner.next(rng) {
            GeneratorState::Yielded(y) => GeneratorState::Yielded((self.closure)(y)),
            GeneratorState::Complete(r) => GeneratorState::Complete(r),
        }
    }
}

/// A generator that yields a clone of a value before another one
/// completes.
///
/// This `struct` is created by the
/// [`suffix`](crate::GeneratorExt::suffix) method on
/// [`Generator`](crate::Generator).
pub type Suffix<G: Generator, EG: Generator<Yield = G::Yield>> = Brace<Empty<G::Yield>, G, EG>;

/// A generator that yields a clone of a value at the beginning of
/// another one.
///
/// This `struct` is created by the
/// [`prefix`](crate::GeneratorExt::suffix) method on
/// [`Generator`](crate::Generator).
pub type Prefix<BG: Generator<Yield = G::Yield>, G: Generator> = Brace<BG, G, Empty<G::Yield>>;

enum BraceState {
    Begin,
    Middle,
    End,
}

/// A generator that is semantically equivalent to
/// `generator.prefix(...).suffix(...)`.
///
/// This `struct` is created by the
/// [`brace`](crate::GeneratorExt::suffix) method on
/// [`Generator`](crate::Generator).
pub struct Brace<BG, G, EG>
where
    BG: Generator,
    G: Generator,
    EG: Generator,
{
    begin: BG,
    inner: G,
    end: EG,
    state: BraceState,
    complete: Option<G::Return>,
}

impl<BG, G, EG> Generator for Brace<BG, G, EG>
where
    BG: Generator<Yield = G::Yield>,
    G: Generator,
    EG: Generator<Yield = G::Yield>,
{
    type Yield = G::Yield;

    type Return = G::Return;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        match self.state {
            BraceState::Begin => match self.begin.next(rng) {
                GeneratorState::Complete(_) => {
                    self.state = BraceState::Middle;
                    self.next(rng)
                }
                GeneratorState::Yielded(y) => GeneratorState::Yielded(y),
            },
            BraceState::Middle => match self.inner.next(rng) {
                GeneratorState::Complete(r) => {
                    self.complete = Some(r);
                    self.state = BraceState::End;
                    self.next(rng)
                }
                GeneratorState::Yielded(y) => GeneratorState::Yielded(y),
            },
            BraceState::End => match self.end.next(rng) {
                GeneratorState::Complete(_) => {
                    self.state = BraceState::Begin;
                    let r = std::mem::replace(&mut self.complete, None).unwrap();
                    GeneratorState::Complete(r)
                }
                GeneratorState::Yielded(y) => GeneratorState::Yielded(y),
            },
        }
    }
}

/// A generator that runs a closure on values generated by another.
///
/// This `struct` is created by the
/// [`inspect`](crate::GeneratorExt::inspect) method on
/// [`Generator`](crate::Generator).
pub struct Inspect<G, F> {
    inner: G,
    closure: F,
}

impl<G, F> Generator for Inspect<G, F>
where
    G: Generator,
    F: Fn(&GeneratorState<G::Yield, G::Return>),
{
    type Yield = G::Yield;

    type Return = G::Return;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        let passthrough = self.inner.next(rng);
        (self.closure)(&passthrough);
        passthrough
    }
}

/// A generator that completes immediately with a clone of a value.
///
/// This `struct` is created by the
/// [`complete`](crate::GeneratorExt::complete) method on
/// [`Generator`](crate::Generator).
pub struct Complete<R>(pub R);

impl<R> Generator for Complete<R>
where
    R: Clone,
{
    type Yield = R;

    type Return = R;

    fn next(&mut self, _rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        GeneratorState::Complete(self.0.clone())
    }
}

/// A wrapper that allows peeking at the next (upcoming) value of a
/// generator without consuming it.
///
/// This `struct` is created by the
/// [`peekable`](crate::GeneratorExt::peekable) method on
/// [`Generator`](crate::Generator).
pub struct Peek<G: Generator> {
    inner: G,
    buffer: VecDeque<GeneratorState<G::Yield, G::Return>>,
}

impl<G> Generator for Peek<G>
where
    G: Generator,
{
    type Yield = G::Yield;

    type Return = G::Return;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        if let Some(next) = self.buffer.pop_front() {
            next
        } else {
            self.inner.next(rng)
        }
    }
}

impl<G> PeekableGenerator for Peek<G>
where
    G: Generator,
{
    fn peek(&mut self, rng: &mut Rng) -> &GeneratorState<G::Yield, G::Return> {
        let next = self.inner.next(rng);
        self.buffer.push_back(next);
        self.buffer.back().unwrap()
    }
}

/// A [`Generator`](crate::Generator) that allows for peeking at the
/// upcoming values without consuming them.
pub trait PeekableGenerator: Generator {
    fn peek(&mut self, rng: &mut Rng) -> &GeneratorState<Self::Yield, Self::Return>;
}

pub struct Concatenate<L: Generator, R: Generator> {
    left: L,
    left_return: Option<L::Return>,
    right: R,
}

impl<L, R> Generator for Concatenate<L, R>
where
    L: Generator,
    R: Generator<Yield = L::Yield>,
{
    type Yield = L::Yield;

    type Return = (L::Return, R::Return);

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        if self.left_return.is_none() {
            match self.left.next(rng) {
                GeneratorState::Complete(r) => {
                    self.left_return = Some(r);
                    self.next(rng)
                }
                GeneratorState::Yielded(y) => GeneratorState::Yielded(y),
            }
        } else {
            match self.right.next(rng) {
                GeneratorState::Complete(right) => {
                    let left = std::mem::replace(&mut self.left_return, None).unwrap();
                    GeneratorState::Complete((left, right))
                }
                GeneratorState::Yielded(y) => GeneratorState::Yielded(y),
            }
        }
    }
}

pub struct Aggregate<G> {
    inner: G,
}

impl<G> Generator for Aggregate<G>
where
    G: Generator,
{
    type Yield = Never;

    type Return = Vec<G::Yield>;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        let mut out = Vec::new();
        while let GeneratorState::Yielded(y) = self.inner.next(rng) {
            out.push(y);
        }
        GeneratorState::Complete(out)
    }
}

pub struct Exhaust<G> {
    inner: G,
}

impl<G> Generator for Exhaust<G>
where
    G: Generator,
{
    type Yield = G::Yield;

    type Return = G::Return;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        GeneratorState::Complete(self.inner.complete(rng))
    }
}

pub struct Chain<G>
where
    G: Generator,
{
    inners: Vec<G>,
    idx: usize,
    completed: Vec<G::Return>,
}

impl<G> Generator for Chain<G>
where
    G: Generator,
{
    type Yield = G::Yield;

    type Return = Vec<G::Return>;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        if self.idx == self.inners.len() {
            let out = std::mem::replace(&mut self.completed, Vec::new());
            self.idx = 0;
            GeneratorState::Complete(out)
        } else {
            let gen = self.inners.get_mut(self.idx).unwrap();
            match gen.next(rng) {
                GeneratorState::Yielded(y) => GeneratorState::Yielded(y),
                GeneratorState::Complete(r) => {
                    self.idx += 1;
                    self.completed.push(r);
                    self.next(rng)
                }
            }
        }
    }
}

impl<G> FromIterator<G> for Chain<G>
where
    G: Generator,
{
    fn from_iter<T: IntoIterator<Item = G>>(iter: T) -> Self {
        Self {
            inners: iter.into_iter().collect(),
            idx: 0,
            completed: Vec::new(),
        }
    }
}

impl<G> Extend<G> for Chain<G>
where
    G: Generator,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = G>,
    {
        self.inners.extend(iter)
    }
}

impl<BG, MG, EG> Extend<MG> for Brace<BG, Chain<MG>, EG>
where
    BG: Generator<Yield = MG::Yield>,
    MG: Generator,
    EG: Generator<Yield = MG::Yield>,
{
    fn extend<T: IntoIterator<Item = MG>>(&mut self, iter: T) {
        self.inner.extend(iter)
    }
}

pub type Just<C> = Once<Constant<C>>;

pub struct Constant<C: Clone>(C);

impl<C: Clone> Constant<C> {
    pub fn new(c: C) -> Constant<C> {
        Constant(c)
    }
}

impl<C> Generator for Constant<C>
where
    C: Clone,
{
    type Yield = C;

    type Return = Never;

    fn next(&mut self, _rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        GeneratorState::Yielded(self.0.clone())
    }
}

pub struct Empty<Y, C = ()>(C, PhantomData<Y>);

impl<Y> Empty<Y> {
    pub fn new() -> Self {
        Self((), PhantomData)
    }
}

impl<Y, C> Empty<Y, C> {
    pub fn complete(c: C) -> Self {
        Self(c, PhantomData)
    }
}

impl<Y, C> Generator for Empty<Y, C>
where
    C: Clone,
{
    type Yield = Y;

    type Return = C;

    fn next(&mut self, _rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        GeneratorState::Complete(self.0.clone())
    }
}

pub struct Labeled<L, I> {
    label: L,
    inner: I,
}

impl<L, G> Generator for Labeled<L, G>
where
    L: Clone,
    G: Generator,
{
    type Yield = Labeled<L, G::Yield>;

    type Return = G::Return;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        match self.inner.next(rng) {
            GeneratorState::Yielded(y) => {
                let labeled = Labeled {
                    label: self.label.clone(),
                    inner: y,
                };
                GeneratorState::Yielded(labeled)
            }
            GeneratorState::Complete(r) => GeneratorState::Complete(r),
        }
    }
}

pub struct Take<G>
where
    G: Generator,
{
    inner: G,
    len: usize,
    rem: usize,
    ret: Vec<G::Return>,
}

impl<G> Generator for Take<G>
where
    G: Generator,
{
    type Yield = G::Yield;

    type Return = ();

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        if self.rem != 0 {
            match self.inner.next(rng) {
                GeneratorState::Yielded(y) => GeneratorState::Yielded(y),
                GeneratorState::Complete(r) => {
                    self.rem -= 1;
                    self.ret.push(r);
                    self.next(rng)
                }
            }
        } else {
            self.rem = self.len;
            let _ret = std::mem::replace(&mut self.ret, Vec::new());
            GeneratorState::Complete(())
        }
    }
}

pub struct Replay<G: Generator> {
    inner: G,
    len: usize,
    rem: Option<usize>,
    idx: usize,
    buf: Vec<G::Yield>,
    ret: Option<G::Return>,
}

impl<G: Generator> Replay<G> {
    pub fn purge(&mut self) {
        self.buf = Vec::new();
        self.ret = None;
        self.idx = 0;

        let len = self.len;
        self.rem.as_mut().map(|rem| *rem = len);
    }
}

impl<G> Generator for Replay<G>
where
    G: Generator,
    G::Yield: Clone,
    G::Return: Clone,
{
    type Yield = G::Yield;

    type Return = G::Return;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        if self.ret.is_some() {
            let mut rem = self.rem;
            if rem.as_ref().map(|rem| *rem > 0).unwrap_or(true) {
                if let Some(next) = self.buf.get(self.idx) {
                    self.idx += 1;
                    GeneratorState::Yielded(next.clone())
                } else {
                    self.idx = 0;
                    rem.as_mut().map(|inner| *inner -= 1);
                    GeneratorState::Complete(self.ret.clone().unwrap())
                }
            } else {
                self.purge();
                self.next(rng)
            }
        } else {
            match self.inner.next(rng) {
                GeneratorState::Yielded(yielded) => {
                    self.buf.push(yielded.clone());
                    GeneratorState::Yielded(yielded)
                }
                GeneratorState::Complete(complete) => {
                    self.ret = Some(complete.clone());
                    GeneratorState::Complete(complete)
                }
            }
        }
    }
}

/// # Panics
/// If `inners` is empty.
pub struct OneOf<G>
where
    G: Generator,
{
    inners: Vec<G>,
    picked: Option<(usize, Box<G>)>,
}

impl<G> FromIterator<G> for OneOf<G>
where
    G: Generator,
{
    fn from_iter<T: IntoIterator<Item = G>>(iter: T) -> Self {
        Self {
            inners: iter.into_iter().collect(),
            picked: None,
        }
    }
}

impl<G> Generator for OneOf<G>
where
    G: Generator,
{
    type Yield = G::Yield;

    type Return = G::Return;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        if let Some((_, picked)) = self.picked.as_mut() {
            match picked.next(rng) {
                GeneratorState::Yielded(y) => GeneratorState::Yielded(y),
                GeneratorState::Complete(r) => {
                    //self.inners.iter_mut().for_each(|others| {
                    //    others.complete(rng);
                    //});
                    let (idx, picked) = std::mem::replace(&mut self.picked, None).unwrap();
                    self.inners.insert(idx, *picked);
                    GeneratorState::Complete(r)
                }
            }
        } else {
            let idx = rng.gen_range(0, self.inners.len());
            self.picked = Some((idx, Box::new(self.inners.remove(idx))));
            self.next(rng)
        }
    }
}

pub struct Maybe<G>
where
    G: Generator,
{
    inner: G,
    include: bool,
}

impl<G> Generator for Maybe<G>
where
    G: Generator,
{
    type Yield = G::Yield;

    type Return = Option<G::Return>;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        if self.include {
            match self.inner.next(rng) {
                GeneratorState::Yielded(y) => GeneratorState::Yielded(y),
                GeneratorState::Complete(r) => {
                    self.include = false;
                    GeneratorState::Complete(Some(r))
                }
            }
        } else {
            self.include = rng.gen();
            if self.include {
                self.next(rng)
            } else {
                GeneratorState::Complete(None)
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[inline]
    pub fn prime<T>(t: T) -> (Constant<T>, Rng)
    where
        T: Clone,
    {
        (Constant::new(t), rand::thread_rng())
    }

    #[test]
    fn once() {
        let (seed, mut rng) = prime(42);
        let mut subject = seed.once();
        assert!(subject.next(&mut rng).is_yielded());
        assert!(subject.next(&mut rng).is_complete());
    }

    #[test]
    fn map() {
        let (seed, mut rng) = prime(42);
        let mut subject = seed.once().map(|value| value - 42);
        assert_eq!(subject.next(&mut rng), GeneratorState::Yielded(42));
        assert_eq!(subject.next(&mut rng), GeneratorState::Complete(0));
    }

    #[test]
    fn and_then() {
        let (seed, mut rng) = prime(42);
        let mut subject = seed
            .once()
            .and_then(|value| Constant::new(value - 42).once());
        assert_eq!(subject.next(&mut rng), GeneratorState::Yielded(42));
        assert_eq!(subject.next(&mut rng), GeneratorState::Yielded(0));
    }

    #[test]
    fn intercept() {
        let (seed, mut rng) = prime(42);
        let mut subject = seed.once().intercept(|value| value - 42);
        assert_eq!(subject.next(&mut rng), GeneratorState::Yielded(0));
        assert_eq!(subject.next(&mut rng), GeneratorState::Complete(42));
    }

    #[test]
    fn concatenate() {
        let (seed, mut rng) = prime(42);
        let mut subject = seed.once().concatenate(Constant::new(84).once());
        assert_eq!(subject.next(&mut rng), GeneratorState::Yielded(42));
        assert_eq!(subject.next(&mut rng), GeneratorState::Yielded(84));
        assert_eq!(subject.next(&mut rng), GeneratorState::Complete((42, 84)));
    }

    #[test]
    fn exhaust() {
        let (seed, mut rng) = prime(42);
        let mut subject = seed.once().exhaust();
        assert_eq!(subject.next(&mut rng), GeneratorState::Complete(42));
    }

    #[test]
    fn brace() {
        let (seed, mut rng) = prime(42);
        let mut subject = seed
            .once()
            .brace(Constant::new(-42).once(), Constant::new(84).once());
        assert_eq!(subject.next(&mut rng), GeneratorState::Yielded(-42));
        assert_eq!(subject.next(&mut rng), GeneratorState::Yielded(42));
        assert_eq!(subject.next(&mut rng), GeneratorState::Yielded(84));
        assert_eq!(subject.next(&mut rng), GeneratorState::Complete(42));
    }

    #[test]
    fn peekable() {
        let (seed, mut rng) = prime(42);
        let mut subject = seed.once().peekable();
        assert_eq!(*subject.peek(&mut rng), GeneratorState::Yielded(42));
        assert_eq!(subject.next(&mut rng), GeneratorState::Yielded(42));
        assert_eq!(*subject.peek(&mut rng), GeneratorState::Complete(42));
    }

    #[test]
    fn aggregate() {
        let (seed, mut rng) = prime(42);
        let mut subject = seed.once().take(5).aggregate();
        assert_eq!(subject.complete(&mut rng), vec![42, 42, 42, 42, 42]);
    }

    #[test]
    fn take() {
        let (seed, mut rng) = prime(42);
        let mut subject = seed.once().take(2);
        assert_eq!(subject.next(&mut rng), GeneratorState::Yielded(42));
        assert_eq!(subject.next(&mut rng), GeneratorState::Yielded(42));
        assert_eq!(
            subject.next(&mut rng),
	    GeneratorState::Complete(())
        );
    }

    #[test]
    fn one_of() {
        let (seed, mut rng) = prime(42i32);
        let mut subject = vec![seed.once()].into_iter().collect::<OneOf<_>>();
        assert_eq!(subject.next(&mut rng), GeneratorState::Yielded(42));
        assert_eq!(subject.next(&mut rng), GeneratorState::Complete(42));
    }

    #[test]
    fn replay() {
        let mut rng = rand::thread_rng();
        let mut gen = Seed::new::<i32>().once().replay(5);
        let mut buf = Vec::new();

        let mut is_complete = false;
        while !is_complete {
            let next = gen.next(&mut rng);
            is_complete = next.is_complete();
            buf.push(next);
        }

        let mut iter_buf = buf.iter();
        let mut restarts = 1;
        while restarts <= 10 {
            let next_buf = match iter_buf.next() {
                Some(next) => next,
                None => {
                    restarts += 1;
                    iter_buf = buf.iter();
                    iter_buf.next().unwrap()
                }
            };
            assert_eq!(*next_buf, gen.next(&mut rng))
        }

        for item in buf {
            assert!(item != gen.next(&mut rng))
        }
    }
}
