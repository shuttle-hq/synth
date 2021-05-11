//! # Rules of the land
//!
//! - New variants added to the `Content` enum need to be of the form
//!   `$variant:ident(${variant}Content)`.
//! - `${variant}Content` has to be exported by a submodule of this.
//! - The submodule must use `super::prelude::*` to use external
//!   imports.
//! - Other content nodes must be imported from this module with
//!   `super::{$content_variant}`.
//! - All foreign API implementations should be in their corresponding
//!   modules.
//! - Public fields should be avoided, short of `content: Content`.
//! - Things that belong to those submodules that also need to be exposed
//!   to other parts of `synth` should be re-exported here.

use serde::{Deserialize, Serialize};
use serde_json::Value;

mod r#bool;
pub use self::r#bool::BoolContent;

mod number;
pub use number::{number_content, NumberContent, NumberContentKind, NumberKindExt, RangeStep};

mod string;
pub use string::{
    ChronoValue, ChronoValueFormatter, ChronoValueType, DateTimeContent, FakerContent,
    FakerContentArgument, RegexContent, StringContent, Uuid,
};

mod array;
pub use array::ArrayContent;

mod object;
pub use object::{FieldContent, ObjectContent};

mod one_of;
pub use one_of::{OneOfContent, VariantContent};

mod categorical;
pub use categorical::{Categorical, CategoricalType};

pub use number::Id;
pub mod prelude;
pub(crate) mod series;

use prelude::*;

use super::FieldRef;
use crate::schema::content::series::SeriesContent;

pub trait Find<C> {
    fn find<I, R>(&self, reference: I) -> Result<&C>
    where
        I: IntoIterator<Item = R>,
        R: AsRef<str>,
    {
        self.project(reference.into_iter().peekable())
    }

    fn find_mut<I, R>(&mut self, reference: I) -> Result<&mut C>
    where
        I: IntoIterator<Item = R>,
        R: AsRef<str>,
    {
        self.project_mut(reference.into_iter().peekable())
    }

    fn project<I, R>(&self, reference: Peekable<I>) -> Result<&C>
    where
        I: Iterator<Item = R>,
        R: AsRef<str>;

    fn project_mut<I, R>(&mut self, reference: Peekable<I>) -> Result<&mut C>
    where
        I: Iterator<Item = R>,
        R: AsRef<str>;
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SameAsContent {
    #[serde(rename = "ref")]
    pub ref_: FieldRef,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum Content {
    Null,
    Bool(BoolContent),
    Number(NumberContent),
    String(StringContent),
    Array(ArrayContent),
    Object(ObjectContent),
    SameAs(SameAsContent),
    OneOf(OneOfContent),
    Series(SeriesContent),
}

impl Content {
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn accepts(&self, value: &Value) -> Result<()> {
        match self {
            Self::SameAs(_) => Ok(()),
            Self::OneOf(one_of_content) => {
                let res: Vec<_> = one_of_content
                    .iter()
                    .map(|content| content.accepts(value))
                    .collect();
                if res.iter().any(|r| r.is_ok()) {
                    Ok(())
                } else {
                    Err(failed!(
                        target: Release,
                        "no variant of this will accept: {}",
                        value
                    ))
                }
            }
            // self is a non-logical node
            _ => match value {
                Value::Null => match self {
                    Self::Null => Ok(()),
                    _ => Err(failed!(
                        target: Release,
                        "expecting: '{}', found: 'null'",
                        value
                    )),
                },
                Value::Bool(_) => match self {
                    Self::Bool(_) => Ok(()),
                    _ => Err(failed!(
                        target: Release,
                        "expecting: '{}', found: 'bool'",
                        self
                    )),
                },
                Value::Number(number_value) => match self {
                    Self::Number(number_content) => number_content.accepts(number_value),
                    _ => Err(failed!(
                        target: Release,
                        "expecting: '{}', found: 'number'",
                        self
                    )),
                },
                Value::String(_) => match self {
                    Self::String(_) => Ok(()),
                    _ => Err(failed!(
                        target: Release,
                        "expecting: '{}', found: 'string'",
                        self
                    )),
                },
                Value::Array(arr) => match self {
                    Self::Array(one_of) => arr
                        .iter()
                        .try_for_each(|value| one_of.content.accepts(value)),
                    _ => Err(failed!(
                        target: Release,
                        "expecting: '{}', found: 'array'",
                        self
                    )),
                },
                Value::Object(obj) => match self {
                    Self::Object(object_content) => object_content.accepts(obj),
                    _ => Err(failed!(
                        target: Release,
                        "expecting: '{}', found: 'object'",
                        self
                    )),
                },
            },
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Content::Null => "null",
            Content::Bool(_) => "bool",
            Content::Number(_) => "number",
            Content::String(_) => "string",
            Content::Array(_) => "array",
            Content::Object(_) => "object",
            Content::SameAs(_) => "same_as",
            Content::OneOf(_) => "one_of",
            Content::Series(_) => "series",
        }
    }
}

impl Default for Content {
    fn default() -> Self {
        Self::Null
    }
}

impl std::fmt::Display for Content {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind())
    }
}

impl<'r> From<&'r Value> for Content {
    fn from(value: &'r Value) -> Self {
        match value {
            // TODO not sure what the correct behaviour is here
            Value::Null => Content::Null,
            Value::Bool(_) => Content::Bool(BoolContent::default()),
            Value::String(_) => Content::String(StringContent::default()),
            Value::Array(arr) => {
                let length = arr.len();
                let one_of_content = arr.iter().collect();
                Content::Array(ArrayContent {
                    length: Box::new(Content::from(&Value::from(length as u64))),
                    content: Box::new(Content::OneOf(one_of_content)),
                })
            }
            Value::Object(obj) => {
                let fields = obj
                    .iter()
                    .map(|(key, value)| (key.to_string(), FieldContent::new(value)))
                    .collect();
                Content::Object(ObjectContent { fields })
            }
            Value::Number(number_value) => {
                let number_content = if number_value.is_f64() {
                    let value = number_value.as_f64().unwrap();
                    NumberContent::F64(number_content::F64::Range(RangeStep {
                        low: value,
                        high: value + 1.0,
                        step: 1.,
                    }))
                } else if number_value.is_u64() {
                    let value = number_value.as_u64().unwrap();
                    NumberContent::U64(number_content::U64::Range(RangeStep {
                        low: value,
                        high: value + 1,
                        step: 1,
                    }))
                } else if number_value.is_i64() {
                    let value = number_value.as_i64().unwrap();
                    NumberContent::I64(number_content::I64::Range(RangeStep {
                        low: value,
                        high: value + 1,
                        step: 1,
                    }))
                } else {
                    unreachable!()
                };
                Content::Number(number_content)
            }
        }
    }
}

impl Find<Content> for Content {
    fn project<I, R>(&self, mut reference: Peekable<I>) -> Result<&Content>
    where
        I: Iterator<Item = R>,
        R: AsRef<str>,
    {
        match reference.peek() {
            None => Ok(self),
            Some(next) => match self {
                Content::Object(object_content) => object_content.project(reference),
                Content::Array(array_content) => array_content.project(reference),
                Content::OneOf(one_of_content) => one_of_content.project(reference),
                _ => Err(failed!(
                    target: Release,
                    "unexpected field name: {}",
                    next.as_ref()
                )),
            },
        }
    }

    fn project_mut<I, R>(&mut self, mut reference: Peekable<I>) -> Result<&mut Content>
    where
        I: Iterator<Item = R>,
        R: AsRef<str>,
    {
        match reference.peek() {
            None => Ok(self),
            Some(next) => match self {
                Content::Object(object_content) => object_content.project_mut(reference),
                Content::Array(array_content) => array_content.project_mut(reference),
                Content::OneOf(one_of_content) => one_of_content.project_mut(reference),
                _ => Err(failed!(
                    target: Release,
                    "unexpected field name: {}",
                    next.as_ref()
                )),
            },
        }
    }
}

impl Compile for Content {
    fn compile<'a, C: Compiler<'a>>(&'a self, compiler: C) -> Result<Graph> {
        match self {
            Self::Object(object_content) => object_content.compile(compiler),
            Self::Bool(bool_content) => bool_content.compile(compiler),
            Self::String(string_content) => string_content.compile(compiler),
            Self::Number(number_content) => number_content.compile(compiler),
            Self::Array(array_content) => array_content.compile(compiler),
            Self::SameAs(same_as_content) => same_as_content.compile(compiler),
            Self::OneOf(one_of_content) => one_of_content.compile(compiler),
            Self::Series(series_content) => series_content.compile(compiler),
            Self::Null => Ok(Graph::null()),
        }
    }
}

impl Compile for SameAsContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, mut compiler: C) -> Result<Graph> {
        compiler.get(self.ref_.clone())
    }
}

#[inline]
pub fn suggest_closest<R, I>(iter: I, reference: &str) -> Option<String>
where
    I: Iterator<Item = R>,
    R: AsRef<str>,
{
    iter.min_by_key(|key| strsim::levenshtein(reference, key.as_ref()))
        .map(|suggest| format!(", did you mean '{}'?", suggest.as_ref()))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(try_from = "f64")]
pub struct Weight(f64);

impl std::convert::TryFrom<f64> for Weight {
    type Error = anyhow::Error;

    fn try_from(value: f64) -> Result<Self> {
        if 0.0 <= value {
            Ok(Self(value))
        } else {
            Err(failed!(
                target: Release,
                "invalid weight: {}. Weights must be non-negative numbers.",
                value
            ))
        }
    }
}

impl Default for Weight {
    fn default() -> Self {
        Self(1.0)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    lazy_static! {
        pub static ref USER_SCHEMA: Content = from_json!({
            "type": "object",
            "user_id": {
                "type": "number",
                "subtype": "u64",
                "range": {
                    "low": 0,
                    "high": 1_000_000,
                    "step": 1
                }
            },
            "type_": { // checks the underscore hack
                "type": "string",
                "pattern": "user|contributor|maintainer"
            },
            "first_name": {
                "type": "string",
                "faker": {
                    "generator": "first_name"
                }
            },
            "last_name": {
                "type": "string",
                "faker": {
                    "generator": "last_name"
                }
            },
            "address": {
                "type": "object",
                "postcode": {
                    "type": "string",
                    "pattern": "[A-Z]{1}[a-z]{3,6}"
                },
                "country": {
                    "optional": true,
                    "type": "string",
                    "faker": {
                        "generator": "country_code",
                        "representation": "alpha-2"
                    }
                },
                "numbers": {
                    "type": "number",
                    "subtype": "u64",
                    "range": {
                        "low": 0,
                        "high": 1_000_000,
                        "step": 1
                    }
                }
            },
            "friends": {
                "type": "array",
                "length": {
                    "type": "number",
                    "subtype": "u64",
                    "constant": 100
                },
                "content": {
                    "type": "one_of",
                    "variants": [ {
                        "type": "string",
                        "pattern": "[A-Z]{1}[a-z]{3,6}"
                    }, {
                        "type": "number",
                        "subtype": "f64",
                        "range": {
                            "low": -75.2,
                            "high": -11,
                            "step": 0.1
                        }
                    } ]
                }
            }
        });
        static ref USER: serde_json::Value = json!({
            "user_id" : 123,
            "type": "user",
            "first_name" : "John",
            "last_name": "Smith",
            "address" : {
                "postcode": "abc123",
                "numbers": 5
            },
            "friends" : ["just", "kidding", 0.5]
        });
    }

    #[test]
    fn user_schema_accepts() {
        println!("{:#?}", *USER_SCHEMA);
        USER_SCHEMA.accepts(&USER).unwrap()
    }

    #[test]
    fn user_schema_declined_extra_field() {
        let user = json!({
            "user_id" : 123,
            "type" : "contributor",
            "first_name" : "John",
            "last_name": "Smith",
            "address" : {
                "postcode": "abc123",
                "numbers": 5
            },
            "friends" : ["just", "kidding", 0.5],
            "extra_field": "some val" // This field is not part of the schema
        });

        assert!(USER_SCHEMA.accepts(&user).is_err());
    }

    #[test]
    fn user_schema_declined_missing_field() {
        let user = json!({
            "user_id" : 123,
            "type" : "maintainer",
            "first_name" : "John",
            "last_name": "Smith",
            "address" : {
                "postcode": "abc123",
                "numbers": 5
            },
            // missing field `friends`
        });

        assert!(USER_SCHEMA.accepts(&user).is_err());
    }

    #[test]
    fn user_schema_declined_bad_array() {
        let user = json!({
            "user_id" : 123,
            "type" : "user",
            "first_name" : "John",
            "last_name": "Smith",
            "address" : {
                "postcode": "abc123",
                "numbers": 5
            },
            "friends" : ["just", "kidding", 0.5, true] // schema does not support booleans
        });

        assert!(USER_SCHEMA.accepts(&user).is_err());
    }

    macro_rules! assert_idempotent {
	($($inner:tt)*) => {
	    let in_: DateTimeContent = from_json!($($inner)*);
	    let out = serde_json::to_string(&in_).unwrap();
	    assert_eq!(serde_json::from_str::<'_, DateTimeContent>(&out).unwrap(), in_);
	}
    }

    #[test]
    fn datetime_content_serde_idempotence() {
        env_logger::builder().is_test(true).init();
        assert_idempotent!({
            "format": "%Y-%m-%d %H:%M:%S",
        });

        assert_idempotent!({
            "format": "%Y-%m-%d",
        });

        assert_idempotent!({
            "format": "%Y-%m-%dT%H:%M:%S%z",
        });

        assert_idempotent!({
            "format": "%Y-%m-%dT%H:%M:%S%z",
            "begin": "2020-11-05T09:53:10+0500"
        });

        assert_idempotent!({
            "format": "%Y-%m-%dT%H:%M:%S%z",
            "end": "2020-11-05T09:53:10+0000"
        });

        assert_idempotent!({
            "format": "%Y-%m-%dT%H:%M:%S%z",
            "begin": "2020-11-05T09:53:10+0500",
            "end": "2020-11-05T09:53:10+0000"
        });

        assert!(serde_json::from_value::<DateTimeContent>(json!({
            "format": "%Y-%m-%dT%H:%M:%S%z",
            "end": "2020-11-05T09:53:10+0500",
            "begin": "2020-11-05T09:53:10+0000"
        }))
        .is_err())
    }
}
