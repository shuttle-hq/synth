use super::prelude::*;

use rand::distributions::Bernoulli;

derive_generator! {
    yield bool,
    return Never,
    pub enum RandomBool {
	Bernoulli(Random<bool, Bernoulli>),
	Constant(Yield<bool>),
	Categorical(Random<bool, Categorical<bool>>),
    }
}

derive_generator! {
    yield Token,
    return Result<Value, Error>,
    pub struct BoolNode(Valuize<Tokenizer<OnceInfallible<RandomBool>>, bool>);
}

impl From<RandomBool> for BoolNode {
    fn from(inner: RandomBool) -> Self {
        Self(
            inner
		.infallible()
                .try_once()
                .into_token()
                .map_complete(value_from_ok::<bool>)
        )
    }
}
