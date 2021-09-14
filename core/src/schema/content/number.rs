use super::prelude::*;

use super::Categorical;

use serde::{
    de::{Deserialize, Deserializer},
    ser::Serializer,
    Serialize,
};
use crate::graph::number::{RandomU32, RandomF32, RandomI32};

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

// Generate a snake case version of NumberContent variant
// name via `serde`. We use this hack because `serde` case
// conversion functions are internal
macro_rules! serde_convert_case {
    ($identifier:ident,$case:expr) => {{
        #[derive(Serialize)]
        #[serde(rename_all = $case)]
        enum SnakeCaseHelper {
            $identifier,
        }
        // Safety: since we derive `Serialize`, unwrap() shouldn't panic
        // for any identifier that doesn't brake `enum` compilation
        serde_json::to_value(&SnakeCaseHelper::$identifier).unwrap()
    }};
}

macro_rules! number_content {
    {
	$(
	    $ty:ty[$is:ident, $def:ident] as $as:ident {
		$(
		    $(#[$default:meta])?
		    $variant:ident($variant_ty:path),
		)*
	    },
	)*
    } => {
        #[derive(Debug, Clone, PartialEq)]
        pub enum NumberContent {
            $($as(number_content::$as),)*
        }

        impl NumberContent {
            pub fn kind(&self) -> &'static str {
                match self {
                    $($(Self::$as(number_content::$as::$variant(_)) => {
                        concat!(stringify!($as), "::", stringify!($variant))
                    },)*)*
                }
            }

            $(
            pub fn $def() -> Self {
                Self::$as(number_content::$as::Range(RangeStep::default()))
            }

            pub fn $is(&self) -> bool {
                match self {
                Self::$as(_) => true,
                _ => false
                }
            }
            )*
        }

        impl Serialize for NumberContent
        {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                match &self {
                    $(
                        NumberContent::$as(value) => {
                            let mut obj = serde_json::to_value(value).map_err(S::Error::custom)?;
                            let obj_map = obj.as_object_mut().ok_or(S::Error::custom("Object value expected"))?;
                            obj_map.insert("subtype".to_string(), serde_convert_case!($as, "snake_case"));
                            obj_map.serialize(serializer)
                        }
                    )*
                }
            }
        }

        impl<'de> Deserialize<'de> for NumberContent {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let mut v: serde_json::Value = Value::deserialize(deserializer)?;
                let as_object = v.as_object_mut().ok_or(D::Error::custom("Object value expected"))?;
                match as_object.remove("subtype") {
                    Some(subtype) => {
                    $(
                        if subtype == stringify!($ty) {
                            if as_object.is_empty() {
                                Ok(Self::$def())
                            } else {
                                let inner = number_content::$as::deserialize(v).map_err(D::Error::custom)?;
                                Ok(NumberContent::$as(inner))
                            }
                        }
                        else
                    )*
                        {
                            //TODO: generate static array with variant names and pass it to
                            //      Error::unknown_variant()
                            Err(D::Error::unknown_variant(format!("{:?}", subtype).as_str(), &[""]))
                        }
                    }
                    None => {
                        $( if let Ok(inner) = number_content::$as::deserialize(&v) {
                            Ok(NumberContent::$as(inner))
                        } else )* {
                            Err(D::Error::custom("Failed to infer numeric type from its value: try specifying the 'subtype' parameter"))
                        }
                    }
                }
            }
        }

        pub mod number_content {
            use super::{RangeStep, Categorical, NumberContent, Id};
            use serde::{Serialize, Deserialize};

            $(
            #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
            #[serde(rename_all = "snake_case")]
            #[serde(deny_unknown_fields)]
            pub enum $as {
                $($variant($variant_ty),)*
            }

            $(
            impl From<$variant_ty> for $as {
                fn from(value: $variant_ty) -> Self {
                    Self::$variant(value)
                }
            }
            )*

            impl From<$as> for NumberContent {
                fn from(value: $as) -> Self {
                    Self::$as(value)
                }
            }
            )*
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
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
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct RangeStep<N> {
    pub low: Option<N>,
    pub high: Option<N>,
    pub step: Option<N>,
    pub include_low: bool,
    pub include_high: bool
}

impl<N> Default for RangeStep<N> {
    fn default() -> Self {
        Self {
            low: None,
            high: None,
            step: None,
            include_low: true,
            include_high: false
        }
    }
}

impl<N> RangeStep<N> {
    pub fn new(low: N, high: N, step: N) -> Self {
        Self {
            low: Some(low),
            high: Some(high),
            step: Some(step),
            ..Default::default()
        }
    }

    fn bound(value: Option<&N>, inclusive: bool) -> std::ops::Bound<&N> {
        match value {
            Some(n) if inclusive => std::ops::Bound::Included(n),
            Some(n) => std::ops::Bound::Excluded(n),
            None => std::ops::Bound::Unbounded
        }
    }

    fn cast<F, M>(self,  f: F) -> RangeStep<M>
        where
            F: Fn(N) -> M
    {
        self.try_cast::<_, _, std::convert::Infallible>(|value| Ok(f(value))).unwrap()
    }

    fn try_cast<F, M, E>(self, f: F) -> Result<RangeStep<M>, E>
        where
            F: Fn(N) -> Result<M, E>
    {
        Ok(RangeStep::<M> {
            low: self.low.map(&f).transpose()?,
            high: self.high.map(&f).transpose()?,
            step: self.step.map(&f).transpose()?,
            include_low: self.include_low,
            include_high: self.include_high
        })
    }
}

impl<N: Copy> RangeStep<N> {
    pub fn step(&self) -> Option<N> {
        self.step.as_ref().cloned()
    }
}

impl<N> std::ops::RangeBounds<N> for RangeStep<N> {
    fn start_bound(&self) -> std::ops::Bound<&N> {
        RangeStep::bound(self.low.as_ref(), self.include_low)
    }

    fn end_bound(&self) -> std::ops::Bound<&N> {
        RangeStep::bound(self.high.as_ref(), self.include_high)
    }
}

number_content!(
    u32[is_u32, default_u32_range] as U32 {
        #[default]
        Range(RangeStep<u32>),
        Categorical(Categorical<u32>),
        Constant(u32),
        Id(Id),
    },
    u64[is_u64, default_u64_range] as U64 {
        #[default]
        Range(RangeStep<u64>),
        Categorical(Categorical<u64>),
        Constant(u64),
        Id(Id),
    },
    i32[is_i32, default_i32_range] as I32 {
        #[default]
        Range(RangeStep<i32>),
        Categorical(Categorical<i32>),
        Constant(i32),
    },
    i64[is_i64, default_i64_range] as I64 {
        #[default]
        Range(RangeStep<i64>),
        Categorical(Categorical<i64>),
        Constant(i64),
    },
    f64[is_f64, default_f64_range] as F64 {
        #[default]
        Range(RangeStep<f64>),
        Constant(f64),
    },
    f32[is_f32, default_f32_range] as F32 {
        #[default]
        Range(RangeStep<f32>),
        Constant(f32),
    },
);

impl Compile for NumberContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, _compiler: C) -> Result<Graph> {
        let number_node = match self {
            Self::U64(u64_content) => {
                let random_u64 = match u64_content {
                    number_content::U64::Range(range) => RandomU64::range(*range)?,
                    number_content::U64::Categorical(categorical_content) => {
                        RandomU64::categorical(categorical_content.clone())
                    }
                    number_content::U64::Constant(val) => RandomU64::constant(*val),
                    number_content::U64::Id(id) => {
                        let gen = Incrementing::new_at(id.start_at.unwrap_or(1));
                        RandomU64::incrementing(gen)
                    }
                };
                random_u64.into()
            }
            Self::I64(i64_content) => {
                let random_i64 = match i64_content {
                    number_content::I64::Range(range) => RandomI64::range(*range)?,
                    number_content::I64::Categorical(categorical_content) => {
                        RandomI64::categorical(categorical_content.clone())
                    }
                    number_content::I64::Constant(val) => RandomI64::constant(*val),
                };
                random_i64.into()
            }
            Self::F64(f64_content) => {
                let random_f64 = match f64_content {
                    number_content::F64::Range(range) => RandomF64::range(*range)?,
                    number_content::F64::Constant(val) => RandomF64::constant(*val),
                };
                random_f64.into()
            }
            Self::U32(u32_content) => {
                let random_u32 = match u32_content {
                    number_content::U32::Range(range) => RandomU32::range(*range)?,
                    number_content::U32::Categorical(categorical_content) => {
                        RandomU32::categorical(categorical_content.clone())
                    }
                    number_content::U32::Constant(val) => RandomU32::constant(*val),
                    number_content::U32::Id(id) => {
                        // todo fix
                        let gen = Incrementing::new_at(id.start_at.unwrap_or_default() as u32);
                        RandomU32::incrementing(gen)
                    }
                };
                random_u32.into()
            }
            Self::I32(i32_content) => {
                let random_i32 = match i32_content {
                    number_content::I32::Range(range) => RandomI32::range(*range)?,
                    number_content::I32::Categorical(categorical_content) => {
                        RandomI32::categorical(categorical_content.clone())
                    }
                    number_content::I32::Constant(val) => RandomI32::constant(*val),
                };
                random_i32.into()
            }
            Self::F32(f32_content) => {
                let random_f32 = match f32_content {
                    number_content::F32::Range(range) => RandomF32::range(*range)?,
                    number_content::F32::Constant(val) => RandomF32::constant(*val),
                };
                random_f32.into()
            }
        };
        Ok(Graph::Number(number_node))
    }
}

impl RangeStep<u64> {
    pub fn upcast(self, to: NumberContentKind) -> Result<NumberContent> {
        match to {
            NumberContentKind::U64 => Ok(number_content::U64::Range(self).into()),
            NumberContentKind::I64 => {
                let cast = self.try_cast(i64::try_from)?;
                Ok(number_content::I64::Range(cast).into())
            }
            NumberContentKind::F64 => {
                let cast = self.cast(|value| value as f64);
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

impl RangeStep<i64> {
    pub fn upcast(self, to: NumberContentKind) -> Result<NumberContent> {
        match to {
            NumberContentKind::U64 => Err(failed!(
                target: Release,
                "cannot downcast numerical subtypes"
            )),
            NumberContentKind::I64 => Ok(number_content::I64::Range(self).into()),
            NumberContentKind::F64 => {
                let cast = self.cast(|value| value as f64);
                Ok(number_content::F64::Range(cast).into())
            }
        }
    }
}

impl RangeStep<f64> {
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

#[cfg(test)]
pub mod tests {
    use super::*;

    use num::One;

    // TODO: f32 deserializes successfully to `inf` when OOR
    macro_rules! test_number_variants {
        ($($test:ident -> $name:literal $as:ident: $ty:ty $(,)?)+) => { $(
            #[test]
            fn $test() {
                // Inferred
                let number_content_as_json = json!({
                    "range": {
                        "low": <$ty>::MIN,
                        "high": <$ty>::MAX
                    }
                });
                let number_content: NumberContent = serde_json::from_value(number_content_as_json).unwrap();
                assert_eq!(
                    number_content,
                    NumberContent::$as(number_content::$as::Range(RangeStep {
                        low: Some(<$ty>::MIN),
                        high: Some(<$ty>::MAX),
                        step: None
                    }))
                );

                // Specified
                let number_content_as_json = json!({
                    "subtype": $name,
                    "constant": <$ty>::one()
                });
                let number_content: NumberContent = serde_json::from_value(number_content_as_json).unwrap();
                assert_eq!(
                    number_content,
                    NumberContent::$as(number_content::$as::Constant(<$ty>::one()))
                );
            }
        )+ }
    }

    test_number_variants!(
        test_u32 -> "u32" U32: u32,
        test_u64 -> "u64" U64: u64,
        test_i32 -> "i32" I32: i32,
        test_i64 -> "i64" I64: i64,
    );
}
