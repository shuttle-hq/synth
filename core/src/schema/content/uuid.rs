use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct UuidContent {}

impl UuidContent {
    pub fn kind(&self) -> String {
        "uuid".to_string()
    }
}

impl Compile for UuidContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, mut compiler: C) -> Result<Graph> {
        let node = UuidNode();
        Ok(Graph::Uuid(node))
    }
}
