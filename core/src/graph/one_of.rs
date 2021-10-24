use super::prelude::*;

pub struct OneOfNode(OneOf<Graph>);

impl FromIterator<(f64, Graph)> for OneOfNode {
    fn from_iter<T: IntoIterator<Item = (f64, Graph)>>(iter: T) -> Self {
        Self(OneOf::from_iter(iter))
    }
}

impl FromIterator<Graph> for OneOfNode {
    fn from_iter<T: IntoIterator<Item = Graph>>(iter: T) -> Self {
        Self(OneOf::from_iter(
            iter.into_iter().map(|s| (1.0, s)).collect::<Vec<_>>(),
        ))
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
