use crate::graph::prelude::{
    Generator, GeneratorState, Never, OwnedSerializable, Rng, TryAggregate, TryGenerator,
    TryGeneratorExt,
};
use crate::{Error, Graph};
use serde::Serialize;
use std::ops::DerefMut;

// This is not great but I don't want Serialized to be generic
enum SerializerType {
    JSON,
}

pub struct Serialized {
    inner: Box<TryAggregate<Graph>>,
    serializer: SerializerType,
}

impl Serialized {
    pub fn new_json(inner: Graph) -> Self {
        Self {
            inner: Box::new(inner.try_aggregate()),
            serializer: SerializerType::JSON,
        }
    }

    fn serialize<S: Serialize>(&self, s: &S) -> String {
        match self.serializer {
            SerializerType::JSON => {
                format!(
                    "{}",
                    serde_json::to_value(&s).expect("this should always serialize")
                )
            }
        }
    }
}

impl Generator for Serialized {
    type Yield = String;
    type Return = Result<Never, Error>;

    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        match self.inner.deref_mut().try_next_yielded(rng) {
            Ok(y) => {
                let serializable = OwnedSerializable::new(y);
                GeneratorState::Yielded(self.serialize(&serializable))
            }
            Err(e) => GeneratorState::Complete(Err(e)),
        }
    }
}
