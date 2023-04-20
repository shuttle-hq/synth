use super::prelude::*;
use crate::graph::uuid::RandomUuid;

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct UuidContent {}

impl Compile for UuidContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, mut _compiler: C) -> Result<Graph> {
        let random_uuid = RandomUuid{ };
        Ok(Graph::Uuid(random_uuid.into()))
    }
}
