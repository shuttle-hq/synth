//! Everything re-exported to get started quickly.
//!
//! # Example
//! ```
//! use synth_gen::prelude::*;
//! ```

pub use crate::de::Deserializator;
pub use crate::ser::OwnedSerializable;
pub use crate::value::{
    IntoToken, IntoTokenGeneratorExt, Number, Primitive, Special, Token, TokenGenerator,
    TokenGeneratorExt, Tokenizer,
};
pub use crate::{Generator, GeneratorExt, TryGenerator, TryGeneratorExt, FallibleGenerator, FallibleGeneratorExt};
pub use crate::generator::*;

pub use rand::Rng;

pub use serde::{Deserialize, Serialize};

#[cfg(feature = "faker")]
pub use fake::faker;

pub use crate::{GeneratorState, Never};
