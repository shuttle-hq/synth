//! Everything re-exported to get started quickly.
//!
//! # Example
//! ```
//! use synth_gen::prelude::*;
//! ```

pub use crate::de::Deserializator;
pub use crate::generator::*;
pub use crate::ser::OwnedSerializable;
pub use crate::value::{
    IntoToken, IntoTokenGeneratorExt, Number, Primitive, Special, Token, TokenGenerator,
    TokenGeneratorExt, Tokenizer,
};
pub use crate::{
    FallibleGenerator, FallibleGeneratorExt, Generator, GeneratorExt, GeneratorResult, TryGenerator, TryGeneratorExt,
};

pub use rand::Rng;

pub use serde::{Deserialize, Serialize};

#[cfg(feature = "faker")]
pub use fake::faker;

pub use crate::{GeneratorState, Never};
