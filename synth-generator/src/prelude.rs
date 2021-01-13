//! Everything re-exported to get started quickly.
//!
//! # Example
//! ```
//! use synth_generator::prelude::*;
//! ```

pub use crate::ser::OwnedSerializable;
pub use crate::value::{
    IntoToken, IntoTokenGeneratorExt, Number, Primitive, Special, Token, TokenGenerator,
    TokenGeneratorExt,
};
pub use crate::{DiagonalGeneratorExt, Generator, GeneratorExt, Rng, TryGeneratorExt};
pub use rand::thread_rng;
pub use serde::{Deserialize, Serialize};

#[cfg(feature = "faker")]
pub use fake::faker;
