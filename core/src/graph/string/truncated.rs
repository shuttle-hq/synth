use crate::graph::prelude::*;
use crate::graph::{RandomString, StringNode};
use anyhow::Result;

pub struct Truncated {
    len: usize,
    inner: Box<RandomString>,
}

impl Truncated {
    pub(crate) fn new(len: usize, graph: Graph) -> Result<Self> {
        match graph {
            Graph::String(StringNode::String(random_string)) => {
                let unwrapped = random_string.into_inner().into_inner();
                Ok(Self {
                    inner: Box::new(unwrapped),
                    len,
                })
            }
            _ => Err(anyhow!(
                "Truncated generators can only have content of type 'string'."
            )),
        }
    }
}

impl Generator for Truncated {
    type Yield = String;
    type Return = Result<String, Error>;

    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        match self.inner.next(rng) {
            GeneratorState::Yielded(mut s) => {
                s.truncate(self.len);
                GeneratorState::Yielded(s)
            }
            GeneratorState::Complete(r) => match r {
                Ok(mut s) => {
                    s.truncate(self.len);
                    GeneratorState::Complete(Ok(s))
                }
                Err(e) => GeneratorState::Complete(Err(e)),
            },
        }
    }
}
