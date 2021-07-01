//#![feature(try_trait)]
#![forbid(unsafe_code)]
#![allow(clippy::all, type_alias_bounds)]
#![warn(rust_2018_idioms)]
// @brokad: suppressed for now
//#![warn(missing_docs, missing_doc_code_examples)]
#![deny(elided_lifetimes_in_paths)]

//! # bynar
//!
//!

use std::{fmt::Debug, marker::PhantomData};

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

/// The standard [`rand::Rng`](rand::Rng) implementation used by this
/// crate.
pub type Rng = rand::rngs::ThreadRng;

pub mod generator;
pub use generator::{
    random, FallibleGenerator, FallibleGeneratorExt, Generator, GeneratorExt, GeneratorResult,
    TryGenerator, TryGeneratorExt,
};

#[cfg(feature = "faker")]
pub use generator::dummy;

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

/*
impl<Y, R> std::ops::Try for GeneratorState<Y, R>
where
    R: std::ops::Try
{
    type Ok = R::Ok;

    type Error = R::Error;

    fn into_result(self) -> Result<Self::Ok, Self::Error> {
        match self {
        GeneratorState::Complete(r) => r.into_result(),
        GeneratorState::Yielded(_) => todo!()
    }
    }

    fn from_error(v: Self::Error) -> Self {
        Self::Complete(R::from_error(v))
    }

    fn from_ok(v: Self::Ok) -> Self {
        Self::Complete(R::from_ok(v))
    }
}
*/

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

    pub fn map_complete<RR, F: FnOnce(R) -> RR>(self, closure: F) -> GeneratorState<Y, RR> {
        match self {
            GeneratorState::Yielded(y) => GeneratorState::Yielded(y),
            GeneratorState::Complete(r) => GeneratorState::Complete(closure(r)),
        }
    }

    pub fn map_yielded<YY, F: FnOnce(Y) -> YY>(self, closure: F) -> GeneratorState<YY, R> {
        match self {
            GeneratorState::Yielded(y) => GeneratorState::Yielded(closure(y)),
            GeneratorState::Complete(r) => GeneratorState::Complete(r),
        }
    }

    pub fn into_complete(self) -> Result<R, Error> {
        if let GeneratorState::Complete(r) = self {
            Ok(r)
        } else {
            Err(Error::custom("unexpected EOF"))
        }
    }

    pub fn as_ref(&self) -> GeneratorState<&Y, &R> {
        match *self {
            GeneratorState::Yielded(ref y) => GeneratorState::Yielded(y),
            GeneratorState::Complete(ref r) => GeneratorState::Complete(r),
        }
    }
}

impl<Y, R, E> GeneratorState<Y, Result<R, E>> {
    pub fn map_ok<RR, F: FnOnce(R) -> RR>(self, closure: F) -> GeneratorState<Y, Result<RR, E>> {
        self.map_complete(|ret| ret.map(closure))
    }

    pub fn map_err<EE, F: FnOnce(E) -> EE>(self, closure: F) -> GeneratorState<Y, Result<R, EE>> {
        self.map_complete(|ret| ret.map_err(closure))
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

/*
/// A trait extension for [`Generator`](crate::Generator)s that yield
/// and return values of the same type `D`.
pub trait DiagonalGeneratorExt<D>: Generator<Yield = D, Return = D> + Sized {}

impl<D, T> DiagonalGeneratorExt<D> for T where T: Generator<Yield = D, Return = D> {}
*/

/*
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
*/
