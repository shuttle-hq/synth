use crate::compile::Compile;
use crate::graph::prelude::content::prelude::unique::Unique;
use crate::graph::UniqueNode;
use crate::{Compiler, Content, Graph};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
// #[serde(tag = "algorithm")]
pub struct UniqueContent {
    // TODO need to accept different types of unique algos
    pub inner: Box<Content>,
}

impl Compile for UniqueContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, compiler: C) -> Result<Graph> {
        let graph = self.inner.compile(compiler)?;
        let unique_node = UniqueNode::Hash(Unique::new(graph));
        Ok(Graph::Unique(unique_node))
    }
}