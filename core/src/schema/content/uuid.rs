use super::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct UuidContent {}

impl Compile for UuidContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, mut _compiler: C) -> Result<Graph> {
        let node = UuidNode();
        Ok(Graph::Uuid(node))
    }
}
