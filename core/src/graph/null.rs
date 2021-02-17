use super::prelude::*;

pub type NullNode = Valuize<Infallible<JustToken<()>, Error>, ()>;
