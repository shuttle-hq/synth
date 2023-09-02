use super::prelude::*;

use rand_regex::Regex as RandRegex;

pub mod constant;
pub mod faker;
pub mod format;
pub mod serialized;
pub mod sliced;
pub mod truncated;

pub use constant::Constant;
pub use faker::{FakerArgs, Locale, RandFaker};
pub use format::{Format, FormatArgs};
pub use serialized::Serialized;
pub use sliced::Sliced;
pub use truncated::Truncated;

derive_generator! {
    yield String,
    return Result<String, Error>,
    pub enum RandomString {
        Regex(OnceInfallible<Random<String, RandRegex>>),
        Faker(TryOnce<RandFaker>),
        Serialized(TryOnce<Serialized>)
        Categorical(OnceInfallible<Random<String, Categorical<String>>>)
        Format(Format),
        Truncated(Truncated),
        Sliced(Sliced),
        Constant(OnceInfallible<Constant>),
    }
}

impl From<RandFaker> for RandomString {
    fn from(faker: RandFaker) -> Self {
        Self::Faker(faker.try_once())
    }
}

impl From<Serialized> for RandomString {
    fn from(serialized: Serialized) -> Self {
        Self::Serialized(serialized.try_once())
    }
}

impl From<RandRegex> for RandomString {
    fn from(regex: RandRegex) -> Self {
        Self::Regex(Random::new_with(regex).infallible().try_once())
    }
}

impl From<Categorical<String>> for RandomString {
    fn from(cat: Categorical<String>) -> Self {
        Self::Categorical(Random::new_with(cat).infallible().try_once())
    }
}

impl From<Truncated> for RandomString {
    fn from(trunc: Truncated) -> Self {
        Self::Truncated(trunc)
    }
}

impl From<Sliced> for RandomString {
    fn from(sliced: Sliced) -> Self {
        Self::Sliced(sliced)
    }
}

impl From<Constant> for RandomString {
    fn from(const_: Constant) -> Self {
        Self::Constant(const_.infallible().try_once())
    }
}

impl From<Format> for RandomString {
    fn from(format: Format) -> Self {
        RandomString::Format(format)
    }
}

derive_generator! {
    yield Token,
    return Result<Value, Error>,
    pub struct StringNode(Valuize<Tokenizer<RandomString>, String>);
}

impl From<RandomString> for StringNode {
    fn from(value: RandomString) -> Self {
        Self(value.into_token().map_complete(value_from_ok::<String>))
    }
}
