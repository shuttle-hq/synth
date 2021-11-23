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

#[cfg(test)]
mod tests {
    use super::{Generator, GeneratorState, Graph, IterNode, Number, Value};
    use rand::SeedableRng;

    #[test]
    fn generator() {
        let iter = (1..2).into_iter().map(|i| Value::Number(i.into()));

        let mut graph = Graph::Iter(IterNode {
            iter: Box::new(iter),
        });
        let mut seed = rand::rngs::StdRng::seed_from_u64(5);

        assert!(matches!(
            graph.next(&mut seed),
            GeneratorState::Complete(Ok(Value::Number(Number::I32(1))))
        ));

        assert!(matches!(
            graph.next(&mut seed),
            GeneratorState::Complete(Err(_))
        ));
    }

    #[test]
    fn generator_cycle() {
        let iter = (1..2).into_iter().map(|i| Value::Number(i.into())).cycle();

        let mut graph = Graph::Iter(IterNode {
            iter: Box::new(iter),
        });
        let mut seed = rand::rngs::StdRng::seed_from_u64(5);

        assert!(matches!(
            graph.next(&mut seed),
            GeneratorState::Complete(Ok(Value::Number(Number::I32(1))))
        ));

        assert!(matches!(
            graph.next(&mut seed),
            GeneratorState::Complete(Ok(Value::Number(Number::I32(1))))
        ));
    }
}
