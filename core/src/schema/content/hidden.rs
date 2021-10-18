#![allow(clippy::derivable_impls)]

use crate::compile::Compile;
use crate::{Compiler, Content, Graph};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Hash)]
#[serde(deny_unknown_fields)]
pub struct HiddenContent {
    pub content: Box<Content>,
}

impl Compile for HiddenContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, compiler: C) -> Result<Graph> {
        let graph = self.content.compile(compiler)?;
        Ok(Graph::Hidden(Box::new(graph)))
    }
}
