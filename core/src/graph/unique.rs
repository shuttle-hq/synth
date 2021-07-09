use crate::graph::prelude::{
    Error, Generator, GeneratorState, Rng, Token, TryFilterMap, TryGeneratorExt, Value,
};
use crate::Graph;

use std::collections::HashMap;
use std::hash::{BuildHasher, Hash, Hasher};

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
            let mut hasher = seen.hasher().build_hasher();
            value.hash(&mut hasher);
            let hash = hasher.finish();

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
    use std::collections::HashSet;

    const NUM_GENERATED: usize = 1024;

    #[test]
    fn unique_node() {
        let usernames = Graph::String(StringNode::from(RandomString::from(
            RandFaker::new("username", Default::default()).unwrap(),
        )));
        let mut rng = rand::thread_rng();
        let output = UniqueNode::hash(usernames, None)
            .repeat(NUM_GENERATED)
            .complete(&mut rng)
            .into_iter()
            .collect::<Result<HashSet<_>, _>>()
            .unwrap();
        assert_eq!(output.len(), NUM_GENERATED);

        let numbers = Graph::Number(NumberNode::from(
            RandomU64::range(RangeStep {
                low: 0,
                high: NUM_GENERATED as u64,
                step: 1,
            })
                .unwrap(),
        ));
        let output = UniqueNode::hash(numbers, None)
            .repeat(NUM_GENERATED)
            .complete(&mut rng)
            .into_iter()
            .collect::<Result<HashSet<_>, _>>()
            .unwrap();
        assert_eq!(output.len(), NUM_GENERATED);

        let constant = Graph::Number(NumberNode::from(RandomU64::constant(44)));
        let output = UniqueNode::hash(constant)
            .repeat(10)
            .complete(&mut rng)
            .into_iter()
            .collect::<Result<HashSet<_>, _>>();
        assert!(output.is_err());
    }
}
