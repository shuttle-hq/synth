use crate::compile::Compile;
use crate::graph::UniqueNode;
use crate::{Compiler, Content, Graph};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Default, Serialize, PartialEq, Eq, Hash)]
pub enum UniqueAlgorithm {
    Hash { retries: Option<usize> },
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Hash)]
#[serde(deny_unknown_fields)]
pub struct UniqueContent {
    #[serde(default)]
    pub algorithm: UniqueAlgorithm,
    pub content: Box<Content>,
}

impl Compile for UniqueContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, compiler: C) -> Result<Graph> {
        let graph = self.content.compile(compiler)?;
        let node = match self.algorithm {
            UniqueAlgorithm::Hash { retries } => UniqueNode::hash(graph, retries),
        };
        Ok(Graph::Unique(node))
    }
}
