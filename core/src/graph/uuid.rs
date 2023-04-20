use super::prelude::*;
use uuid::Uuid;

derive_generator! {
    yield Token,
    return Result<Value, Error>,
    pub struct UuidNode(Valuize<Tokenizer<RandomUuid>, Uuid>);
}

impl From<RandomUuid> for UuidNode {
    fn from(value: RandomUuid) -> Self {
        Self(
            value
                .into_token()
                .map_complete(value_from_ok::<Uuid>),
            )
        }
}

pub struct RandomUuid {}

impl Generator for RandomUuid {
    type Yield = Token;
    type Return = Result<Uuid, Error>;

    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        let uuid = Uuid::from_u128(rng.gen());
        GeneratorState::Complete(Ok(uuid))
    }
}
