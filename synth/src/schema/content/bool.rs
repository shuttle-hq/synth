use super::prelude::*;

use super::Categorical;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum BoolContent {
    Frequency(f64),
    Constant(bool),
    Categorical(Categorical<bool>),
}

impl BoolContent {
    pub fn kind(&self) -> &str {
        match self {
            Self::Frequency(_) => "frequency",
            Self::Constant(_) => "constant",
            Self::Categorical(_) => "categorical",
        }
    }
}

impl Default for BoolContent {
    fn default() -> Self {
        Self::Frequency(0.5)
    }
}

impl Compile for BoolContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, _compiler: C) -> Result<Model> {
        let bool_model = match self {
            BoolContent::Frequency(p) => {
                let distr = Bernoulli::new(*p).map_err(|err| {
                    failed!(target: Release, "invalid frequency: p = '{}'", p).context(err)
                })?;
                let seed = Seed::new_with(distr).once().into_token();
                BoolModel::Bernoulli(seed)
            }
            BoolContent::Constant(value) => BoolModel::Constant(value.yield_token()),
            BoolContent::Categorical(categorical_content) => {
                let gen = Seed::new_with(categorical_content.clone().into())
                    .once()
                    .into_token();
                BoolModel::Categorical(gen)
            }
        };
        Ok(Model::Primitive(PrimitiveModel::Bool(bool_model)))
    }
}
