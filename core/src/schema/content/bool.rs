use super::prelude::*;
use std::hash::Hasher;

use super::Categorical;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum BoolContent {
    Frequency(f64),
    Constant(bool),
    Categorical(Categorical<bool>),
}

impl BoolContent {
    pub fn kind(&self) -> String {
        match self {
            Self::Frequency(_) => "frequency".to_string(),
            Self::Constant(_) => "constant".to_string(),
            Self::Categorical(_) => "categorical".to_string(),
        }
    }
}

impl Default for BoolContent {
    fn default() -> Self {
        Self::Frequency(0.5)
    }
}

impl Compile for BoolContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, _compiler: C) -> Result<Graph> {
        let random_bool = match self {
            BoolContent::Frequency(p) => {
                let distr = Bernoulli::new(*p).map_err(|err| {
                    failed!(target: Release, "invalid frequency: p = '{}'", p).context(err)
                })?;
                RandomBool::Bernoulli(Random::new_with(distr))
            }
            BoolContent::Constant(value) => RandomBool::Constant(Yield::wrap(*value)),
            BoolContent::Categorical(categorical_content) => {
                RandomBool::Categorical(Random::new_with(categorical_content.clone()))
            }
        };
        Ok(Graph::Bool(random_bool.into()))
    }
}

impl Hash for BoolContent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Frequency(f) => f.to_bits().hash(state),
            Self::Categorical(c) => c.hash(state),
            Self::Constant(c) => c.hash(state),
        }
    }
}

impl PartialEq for BoolContent {
    fn eq(&self, other: &BoolContent) -> bool {
        match (self, other) {
            (Self::Frequency(f), Self::Frequency(of)) => f == of,
            (Self::Categorical(c), Self::Categorical(oc)) => c == oc,
            (Self::Constant(c), Self::Constant(oc)) => c == oc,
            _ => false,
        }
    }
}
