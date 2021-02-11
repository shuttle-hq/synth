use super::prelude::*;

use super::Categorical;
use crate::gen::IncrementingSeed;
use num::Zero;

#[derive(Clone, Copy)]
pub enum NumberContentKind {
    U64,
    I64,
    F64,
}

impl NumberContentKind {
    pub fn upcast(self) -> Self {
        match self {
            Self::U64 => Self::I64,
            Self::I64 => Self::F64,
            Self::F64 => Self::F64,
        }
    }
}

pub trait NumberKindExt {
    fn kind(&self) -> NumberContentKind;
}

impl NumberKindExt for Number {
    fn kind(&self) -> NumberContentKind {
        if self.is_u64() {
            NumberContentKind::U64
        } else if self.is_i64() {
            NumberContentKind::I64
        } else if self.is_f64() {
            NumberContentKind::F64
        } else {
            unreachable!()
        }
    }
}

macro_rules! number_content {
    {
	$(
	    $ty:ty[$is:ident, $def:ident] as $as:ident {
		$(
		    $(#[$default:meta])?
		    $variant:ident($variant_ty:ty),
		)*
	    },
	)*
    } => {
	#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
	#[serde(rename_all = "snake_case")]
	#[serde(tag = "subtype")]
	#[serde(deny_unknown_fields)]
	pub enum NumberContent {
	    $(
		$as(number_content::$as),
	    )*
	}

	pub mod number_content {
	    use super::{Range, Categorical, NumberContent};
	    use serde::{Serialize, Deserialize};

	    $(
		#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
		#[serde(rename_all = "snake_case")]
		#[serde(deny_unknown_fields)]
		pub enum $as {
		    $(
			$variant($variant_ty),
		    )*
		}

		impl From<$as> for NumberContent {
		    fn from(value: $as) -> Self {
			Self::$as(value)
		    }
		}
	    )*
	}

	#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
	#[serde(deny_unknown_fields)]
    pub struct Id {
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub start_at: Option<u64>,
    }

	impl NumberContent {
	    pub fn accepts(&self, number: &Number) -> Result<()> {
		if self.is_u64() && number.is_u64()
                    || self.is_i64() && number.is_i64()
                    || self.is_f64() && number.is_f64()
                {
                    Ok(())
                } else {
		    // TODO: better error
                    Err(failed!(target: Release, "numerical type mismatch"))
                }
	    }

	    pub fn kind(&self) -> &'static str {
		match self {
		    $(
			$(
			    Self::$as(number_content::$as::$variant(_)) => {
				concat!(stringify!($as), "::", stringify!($variant))
			    },
			)*
		    )*
		}
	    }

	    $(
		pub fn $def() -> Self {
		    Self::$as(number_content::$as::Range(Range::default()))
		}

		pub fn $is(&self) -> bool {
		    match self {
			Self::$as(_) => true,
			_ => false
		    }
		}
	    )*
	}
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Range<N: PartialOrd + Zero + Display> {
    pub low: N,
    pub high: N,
    pub step: N,
}

impl<N: PartialOrd + Zero + Display> Range<N> {
    #[allow(dead_code)]
    pub(crate) fn new(low: N, high: N, step: N) -> Self {
        Self { low, high, step }
    }
}

impl<N: PartialOrd + Zero + Display> Default for Range<N>
where
    N: Bounded + One,
{
    fn default() -> Self {
        Self {
            low: N::min_value(),
            high: N::max_value(),
            step: N::one(),
        }
    }
}

number_content!(
    u64[is_u64, default_u64_range] as U64 {
    #[default]
    Range(Range<u64>),
    Categorical(Categorical<u64>),
    Constant(u64),
    Id(crate::schema::Id),
    },
    i64[is_i64, default_i64_range] as I64 {
    #[default]
    Range(Range<i64>),
    Categorical(Categorical<i64>),
    Constant(i64),
    },
    f64[is_f64, default_f64_range] as F64 {
    #[default]
    Range(Range<f64>),
    Constant(f64),
    },
);

impl Compile for NumberContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, _compiler: C) -> Result<Model> {
        let number_model = match self {
            Self::U64(u64_content) => match u64_content {
                number_content::U64::Range(range) => NumberModel::u64_range(range.clone())?,
                number_content::U64::Categorical(categorical_content) => {
                    let gen = Seed::new_with(categorical_content.clone().into())
                        .once()
                        .into_token();
                    NumberModel::U64Categorical(gen)
                }
                number_content::U64::Constant(val) => NumberModel::u64_constant(*val),
                number_content::U64::Id(id) => {
                    let gen = IncrementingSeed {
                        count: id.start_at.unwrap_or(0),
                    };
                    NumberModel::U64Id(gen.once().into_token())
                }
            },
            Self::I64(i64_content) => match i64_content {
                number_content::I64::Range(range) => NumberModel::i64_range(range.clone())?,
                number_content::I64::Categorical(categorical_content) => {
                    let gen = Seed::new_with(categorical_content.clone().into())
                        .once()
                        .into_token();
                    NumberModel::I64Categorical(gen)
                }
                number_content::I64::Constant(val) => NumberModel::i64_constant(*val),
            },
            Self::F64(f64_content) => match f64_content {
                number_content::F64::Range(range) => NumberModel::f64_range(range.clone())?,
                number_content::F64::Constant(val) => NumberModel::f64_constant(*val),
            },
        };
        Ok(Model::Primitive(PrimitiveModel::Number(number_model)))
    }
}

impl Range<u64> {
    pub fn upcast(self, to: NumberContentKind) -> Result<NumberContent> {
        match to {
            NumberContentKind::U64 => Ok(number_content::U64::Range(self).into()),
            NumberContentKind::I64 => {
                let cast = Range {
                    low: i64::try_from(self.low)?,
                    high: i64::try_from(self.high)?,
                    step: i64::try_from(self.step)?,
                };
                Ok(number_content::I64::Range(cast).into())
            }
            NumberContentKind::F64 => {
                let cast = Range {
                    low: self.low as f64,
                    high: self.high as f64,
                    step: self.step as f64,
                };
                Ok(number_content::F64::Range(cast).into())
            }
        }
    }
}

impl Categorical<u64> {
    pub fn upcast(self, to: NumberContentKind) -> Result<NumberContent> {
        match to {
            NumberContentKind::U64 => {
		Ok(number_content::U64::Categorical(self).into())
	    }
            NumberContentKind::I64 => {
		let cast = Categorical {
		    seen: self
			.seen
			.into_iter()
			.map(|(k, v)| {
			    i64::try_from(k)
				.map(|k_cast| (k_cast, v))
				.map_err(|err| err.into())
			}).collect::<Result<_>>()?,
		    total: self.total
		};
		Ok(number_content::I64::Categorical(cast).into())
	    }
            NumberContentKind::F64 => Err(failed!(target: Release, "cannot upcast categorical subtypes to accept floats; try changing this another numerical subtype manually"))
        }
    }
}

impl Range<i64> {
    pub fn upcast(self, to: NumberContentKind) -> Result<NumberContent> {
        match to {
            NumberContentKind::U64 => Err(failed!(
                target: Release,
                "cannot downcast numerical subtypes"
            )),
            NumberContentKind::I64 => Ok(number_content::I64::Range(self).into()),
            NumberContentKind::F64 => {
                let cast = Range {
                    low: self.low as f64,
                    high: self.high as f64,
                    step: self.step as f64,
                };
                Ok(number_content::F64::Range(cast).into())
            }
        }
    }
}

impl Range<f64> {
    pub fn upcast(self, to: NumberContentKind) -> Result<NumberContent> {
        match to {
            NumberContentKind::U64 | NumberContentKind::I64 => Err(failed!(
                target: Release,
                "cannot downcast numerical subtypes"
            )),
            NumberContentKind::F64 => Ok(number_content::F64::Range(self).into()),
        }
    }
}

impl Categorical<i64> {
    pub fn upcast(self, to: NumberContentKind) -> Result<NumberContent> {
        match to {
            NumberContentKind::U64 => {
                Err(failed!(target: Release, "cannot downcast numerical subtypes"))
            }
            NumberContentKind::I64 => Ok(number_content::I64::Categorical(self).into()),
            NumberContentKind::F64 => Err(failed!(target: Release, "cannot upcast categorical subtypes to accept floats; try changing this another numerical subtype manually")),
        }
    }
}

impl number_content::U64 {
    pub fn upcast(self, to: NumberContentKind) -> Result<NumberContent> {
        match self {
            Self::Range(range) => range.upcast(to),
            Self::Categorical(cat) => cat.upcast(to),
            Self::Constant(val) => match to {
                NumberContentKind::U64 => Ok(self.into()),
                NumberContentKind::I64 => {
                    let cast = i64::try_from(val)?;
                    Ok(number_content::I64::Constant(cast).into())
                }
                NumberContentKind::F64 => {
                    let cast = val as f64;
                    Ok(number_content::F64::Constant(cast).into())
                }
            },
            Self::Id(_id) => Err(failed!(
                target: Release,
                "cannot upcast an id number subtype: only unsigned integers are supported"
            )),
        }
    }
}

impl number_content::I64 {
    pub fn upcast(self, to: NumberContentKind) -> Result<NumberContent> {
        match self {
            Self::Range(range) => range.upcast(to),
            Self::Categorical(cat) => cat.upcast(to),
            Self::Constant(val) => match to {
                NumberContentKind::U64 => Err(failed!(
                    target: Release,
                    "cannot downcast numerical subtypes"
                )),
                NumberContentKind::I64 => Ok(self.into()),
                NumberContentKind::F64 => {
                    let cast = val as f64;
                    Ok(number_content::F64::Constant(cast).into())
                }
            },
        }
    }
}

impl number_content::F64 {
    pub fn upcast(self, to: NumberContentKind) -> Result<NumberContent> {
        match self {
            Self::Range(range) => range.upcast(to),
            Self::Constant(_) => match to {
                NumberContentKind::U64 => Err(failed!(
                    target: Release,
                    "cannot downcast numerical subtypes"
                )),
                NumberContentKind::I64 => Err(failed!(
                    target: Release,
                    "cannot downcast numerical subtypes"
                )),
                NumberContentKind::F64 => Ok(self.into()),
            },
        }
    }
}
