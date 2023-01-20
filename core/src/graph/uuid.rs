use super::prelude::*;
use uuid::Uuid;

pub struct UuidNode();

impl Generator for UuidNode {
    type Yield = Token;
    type Return = Result<Value, Error>;

    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        GeneratorState::Complete(Ok(Value::Uuid(Uuid::from_u128(rng.gen()))))
    }
}
