use crate::graph::prelude::{Error, Rng, Token, TryFilterMap, TryGeneratorExt, Value};
use crate::Graph;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

const MAX_RETRIES: usize = 64;

type ValueFilter =
    TryFilterMap<Box<Graph>, Box<dyn FnMut(Value) -> Result<Option<Value>, Error>>, Value>;

derive_generator! {
    yield Token,
    return Result<Value, Error>,
    pub enum UniqueNode {
        Hash(ValueFilter),
    }
}

impl UniqueNode {
    pub fn hash(inner: Graph, retries: Option<usize>) -> Self {
        let mut seen: HashMap<u64, usize> = HashMap::new();
        let filter = move |value: Value| {
            let hash = seen.hasher().hash_one(&value);

            let count = seen
                .entry(hash)
                .and_modify(|i| {
                    *i += 1;
                })
                .or_insert(0);

            match *count {
                0 => Ok(Some(value)),
                x if x < retries.unwrap_or(MAX_RETRIES) => Ok(None),
                _ => Err(failed_crate!(
                    target: Release,
                    "Could not generate enough unique values from generator: \
                    try reducing the number of values generated"
                )),
            }
        };
        Self::Hash(Box::new(inner).try_filter_map(Box::new(filter)))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::graph::{
        prelude::{Generator, GeneratorExt},
        Graph, NumberNode, RandFaker, RandomString, RandomU64, RangeStep, StringNode,
    };

    const NUM_GENERATED: usize = 1024;

    #[test]
    fn unique_node() {
        let usernames = Graph::String(StringNode::from(RandomString::from(
            RandFaker::new("username", Default::default()).unwrap(),
        )));
        let mut rng = rand::thread_rng();
        let output = UniqueNode::hash(usernames, None)
            .repeat(NUM_GENERATED)
            .complete(&mut rng);

        assert!(output.iter().all(Result::is_ok));
        assert_eq!(output.len(), NUM_GENERATED);

        let numbers = Graph::Number(NumberNode::from(
            RandomU64::range(RangeStep::new(0, NUM_GENERATED as u64, 1)).unwrap(),
        ));
        let output = UniqueNode::hash(numbers, None)
            .repeat(NUM_GENERATED)
            .complete(&mut rng);

        assert!(output.iter().all(Result::is_ok));
        assert_eq!(output.len(), NUM_GENERATED);

        let constant = Graph::Number(NumberNode::from(RandomU64::constant(44)));
        let output = UniqueNode::hash(constant, None)
            .repeat(10)
            .complete(&mut rng);

        assert!(output.iter().any(Result::is_err));
    }
}
