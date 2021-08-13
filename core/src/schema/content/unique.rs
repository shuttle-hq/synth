use crate::compile::Compile;
use crate::graph::UniqueNode;
use crate::{Compiler, Content, Graph};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[bindlang::bindlang]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum UniqueAlgorithm {
    Hash { retries: Option<usize> },
}

#[bindlang::bindlang]
impl Default for UniqueAlgorithm {
    fn default() -> Self {
        Self::Hash { retries: None }
    }
}

#[bindlang::bindlang]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct UniqueContent {
    #[serde(default)]
    pub algorithm: UniqueAlgorithm,
    pub content: Box<Content>,
}

/// Make a unique content of some content
#[bindlang::bindlang]
pub fn unique(content: Content) -> Content {
    Content::Unique(UniqueContent {
        algorithm: UniqueAlgorithm::default(),
        content: Box::new(content)
    })
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
