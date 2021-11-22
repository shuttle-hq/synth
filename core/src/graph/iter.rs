use super::prelude::*;

/// A special [Graph] node for any type of [Content] that will produce an iterator of [Value]
/// The special [Iterator::cycle()] can be used at the construction side when this needs to produce values endlessly.
pub struct IterNode {
    pub iter: Box<dyn Iterator<Item = Value>>,
}

impl Generator for IterNode {
    type Yield = Token;
    type Return = Result<Value, Error>;

    fn next<R: Rng>(&mut self, _rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        if let Some(item) = self.iter.next() {
            GeneratorState::Complete(Ok(item))
        } else {
            GeneratorState::Complete(Err(failed_crate!(
                target: Release,
                "ran out of items for iterator node. Consider setting '\"cycle\": true`"
            )))
        }
    }
}
