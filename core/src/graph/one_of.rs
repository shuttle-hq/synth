use super::prelude::*;

pub struct OneOfNode(OneOf<Graph>);

impl FromIterator<Graph> for OneOfNode {
    fn from_iter<T: IntoIterator<Item = Graph>>(iter: T) -> Self {
        Self(OneOf::from_iter(iter))
    }
}

impl Generator for OneOfNode {
    type Yield = Token;

    type Return = Result<Value, Error>;

    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        self.0
            .next(rng)
            .map_complete(|m_n| m_n.unwrap_or(Ok(Value::Null(()))))
    }
}
