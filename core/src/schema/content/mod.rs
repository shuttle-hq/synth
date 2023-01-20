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

#![allow(clippy::assertions_on_result_states)]
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

use paste::paste;
use serde::{de::IntoDeserializer, Deserialize, Serialize};
use serde_json::Value;

mod r#bool;
pub use self::r#bool::BoolContent;

mod number;
pub use number::{number_content, NumberContent, NumberContentKind, NumberKindExt, RangeStep};

mod string;
pub use string::{
    ConstantContent, FakerContent, FakerContentArgument, FormatContent, RegexContent,
    SlicedContent, StringContent,
};

mod date_time;
pub use date_time::{
    ChronoValue, ChronoValueAndFormat, ChronoValueFormatter, ChronoValueType, DateTimeContent,
};

mod array;
pub use array::ArrayContent;

mod object;
pub use object::ObjectContent;

mod datasource;
pub use datasource::DatasourceContent;

mod one_of;
pub use one_of::{OneOfContent, VariantContent};

mod categorical;
pub use categorical::{Categorical, CategoricalType};

pub use number::Id;
pub mod prelude;

pub mod series;
pub use series::SeriesContent;

pub mod unique;
pub use unique::{UniqueAlgorithm, UniqueContent};

pub mod hidden;
pub use hidden::HiddenContent;

use prelude::*;

use super::{FieldRef, Namespace};

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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct SameAsContent {
    #[serde(rename = "ref")]
    pub ref_: FieldRef,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct NullContent;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct EmptyContent;

#[derive(Serialize, Deserialize, Debug)]
pub struct ContentLabels {
    #[serde(default)]
    optional: bool,
    #[serde(default)]
    unique: bool,
    #[serde(default)]
    hidden: bool,
}

impl ContentLabels {
    fn try_wrap<E>(self, content: Content) -> std::result::Result<Content, E>
    where
        E: serde::de::Error,
    {
        let mut output = content;

        if self.unique {
            output = output.into_unique();
        }

        if self.optional {
            output = output.into_nullable();
        }

        if self.hidden {
            output = output.into_hidden();
        }

        Ok(output)
    }
}

lazy_static! {
    static ref UNEXPECTED: BTreeMap<&'static str, Vec<&'static str>> = {
        let mut m = BTreeMap::new();
        m.insert("arguments", vec!["format"]);
        m.insert("low", vec!["range"]);
        m.insert("high", vec!["range"]);
        m.insert("step", vec!["range"]);
        m.insert("include_high", vec!["range"]);
        m.insert("include_low", vec!["range"]);
        m.insert("start_at", vec!["id"]);
        m.insert("start", vec!["incrementing", "poisson", "cyclical"]);
        m.insert("increment", vec!["incrementing"]);
        m.insert("rate", vec!["poisson"]);
        m.insert("min_rate", vec!["cyclical"]);
        m.insert("max_rate", vec!["cyclical"]);
        m.insert("period", vec!["cyclical"]);
        m.insert("series", vec!["zip"]);
        m
    };
}

macro_rules! content {
    {
        labels: $labels:ty,
        variants: {
            $($name:ident($variant:ty) => $msg:tt,)+
        }
    } => {
        #[derive(Debug, Serialize, Clone, PartialEq, Hash)]
        #[serde(rename_all = "snake_case")]
        #[serde(tag = "type")]
        #[serde(deny_unknown_fields)]
        pub enum Content {
            $($name($variant),)+
        }

        impl<'de> Deserialize<'de> for Content {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                #[derive(Deserialize)]
                #[serde(rename_all = "snake_case")]
                #[serde(tag = "type")]
                #[serde(deny_unknown_fields)]
                enum __Content {
                    $($name($variant),)+
                }

                #[derive(Deserialize)]
                struct __ContentWithLabels {
                    #[serde(flatten)]
                    labels: $labels,
                    #[serde(flatten)]
                    content: __Content
                }

                struct Visitor;

                impl<'de> serde::de::Visitor<'de> for Visitor {
                    type Value = Content;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(formatter, "a general content node, a literal string starting with `@` (for a reference), or a JSON number (for a constant)")
                    }

                    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::MapAccess<'de>
                    {
                        use serde::de::IntoDeserializer;
                        let mut out = HashMap::<String, serde_json::Value>::new();
                        while let Some(key) = map.next_key()? {
                            let value = map.next_value()?;
                            out.insert(key, value);
                        }

                        if out.is_empty() {
                            out.insert("type".to_string(), serde_json::Value::String("empty".to_string()));
                        } else if out.len() == 1 && out.contains_key("type") {
                            paste! {
                            match out.get("type").unwrap().as_str() {
                                $(
                                    Some(stringify!([<$name:snake>])) => generator_field_error!($name, $msg),
                                )*
                                None | Some(_) => {}
                            }
                            }
                        }

                        for key in out.keys() {
                            if let Some(parent) = UNEXPECTED.get(key as &str) {
                                let parents = parent.iter().map(|p| format!("`{}`", p)).collect::<Vec<String>>().join(", ");
                                return Err(A::Error::custom(format!("`{}` is expected to be a field of {}", key, parents)));
                            }
                        }

                        let __ContentWithLabels { labels, content } = __ContentWithLabels::deserialize(out.into_deserializer()).map_err(A::Error::custom)?;
                        match content {
                            $(
                                __Content::$name(inner) => {
                                    let inner_as_content = Content::$name(inner);
                                    labels.try_wrap(inner_as_content)
                                },
                            )+
                        }
                    }

                    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error
                    {
                        Ok(Content::Number(number_content::U64::from(v).into()))
                    }

                    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error
                    {
                        Ok(Content::Number(number_content::I64::from(v).into()))
                    }

                    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error
                    {
                        Ok(Content::Number(number_content::F64::from(v).into()))
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error
                    {
                        if let Some(s) = v.strip_prefix("@") {
                            let ref_ = FieldRef::deserialize(s.into_deserializer())?;
                            Ok(Content::SameAs(SameAsContent { ref_ }))
                        } else {
                            Ok(Content::String(StringContent::Constant(ConstantContent::from(v.to_string()))))
                        }
                    }
                }

                deserializer.deserialize_any(Visitor)
            }
        }
    }
}
macro_rules! generator_field_error {
    ($name:ident, None) => {
        {}
    };
    ($name:ident, $msg:tt) => {
        paste! {
            return Err(A::Error::custom(concat!("`", stringify!([<$name:snake>]), "` generator is ", $msg)))
        }
    }
}

content! {
    labels: ContentLabels,
    variants: {
        Null(NullContent) => None,
        Bool(BoolContent) => "missing a subtype. Try adding `constant`, or `frequency`",
        Number(NumberContent) => "missing a subtype. Try adding `constant`, `range`, or `id`",
        String(StringContent) => "missing a subtype. Try adding `pattern`, `faker`, `categorical`, `serialized`, `uuid`, `truncated`, or `format`",
        DateTime(DateTimeContent) => "missing a `format` field",
        Array(ArrayContent) => "missing a `length` and `content` field",
        Object(ObjectContent) => None,
        SameAs(SameAsContent) => "missing a `ref` field",
        OneOf(OneOfContent) => "missing a `variants` field",
        Series(SeriesContent) => "missing a variant. Try adding `incrementing`, `poisson`, `cyclical`, or `zip`",
        Unique(UniqueContent) => "missing a `content` field",
        Datasource(DatasourceContent) => "missing a `path` field",
        Hidden(HiddenContent) => "missing a `content` field",
        Empty(EmptyContent) => None,
    }
}

impl Content {
    pub fn from_value_wrapped_in_array(value: &Value) -> Self {
        Content::Array(ArrayContent {
            length: Box::new(Content::from(&Value::from(1))),
            content: Box::new(value.into()),
        })
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null(_))
    }

    pub fn null() -> Self {
        Self::Null(NullContent)
    }

    pub fn as_nullable(&self) -> Option<&Self> {
        match self {
            Self::OneOf(one_of) => one_of.as_nullable(),
            _ => None,
        }
    }

    pub fn is_nullable(&self) -> bool {
        self.as_nullable().is_some()
    }

    #[must_use]
    pub fn into_nullable(self) -> Self {
        if !self.is_nullable() {
            Content::OneOf(vec![self, Content::null()].into_iter().collect())
        } else {
            self
        }
    }

    #[must_use]
    pub fn into_hidden(self) -> Self {
        if !self.is_hidden() {
            Content::Hidden(HiddenContent {
                content: Box::new(self),
            })
        } else {
            self
        }
    }

    pub fn is_hidden(&self) -> bool {
        matches!(self, Self::Hidden(_))
    }

    pub fn is_unique(&self) -> bool {
        matches!(self, Self::Unique(_))
    }

    pub fn is_scalar(&self, ns: &Namespace) -> Result<bool> {
        match self {
            Self::Array(_) | Self::Object(_) => Ok(false),
            Self::SameAs(same_as) => ns.get_s_node(&same_as.ref_)?.is_scalar(ns),
            Self::OneOf(one_of) => {
                for variant in &one_of.variants {
                    if !variant.content.is_scalar(ns)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Self::Unique(unique) => unique.content.is_scalar(ns),
            _ => Ok(true),
        }
    }

    #[must_use]
    pub fn into_unique(self) -> Self {
        if !self.is_unique() {
            Content::Unique(UniqueContent {
                algorithm: UniqueAlgorithm::default(),
                content: Box::new(self),
            })
        } else {
            self
        }
    }

    pub fn into_namespace(self) -> Result<Namespace> {
        match self {
            Content::Object(ObjectContent { fields, .. }) => {
                let mut namespace = Namespace::new();
                for (key, content) in fields.into_iter() {
                    namespace.put_collection(key, content)?;
                }
                Ok(namespace)
            }
            _ => Err(anyhow!(
                "cannot convert a non-object content to a namespace"
            )),
        }
    }

    pub fn accepts(&self, value: &Value) -> Result<()> {
        match self {
            Self::Unique(unique_content) => unique_content.content.accepts(value),
            Self::Hidden(_) => Ok(()),
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
                    Self::Null(_) => Ok(()),
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

    pub fn kind(&self) -> String {
        match self {
            Content::Null(_) => "null".to_string(),
            Content::Bool(content) => format!("bool::{}", content.kind()),
            Content::Number(content) => format!("number::{}", content.kind()),
            Content::String(content) => format!("string::{}", content.kind()),
            Content::DateTime(_) => "date_time".to_string(),
            Content::Array(_) => "array".to_string(),
            Content::Object(_) => "object".to_string(),
            Content::SameAs(_) => "same_as".to_string(),
            Content::OneOf(_) => "one_of".to_string(),
            Content::Series(content) => format!("series::{}", content.kind()),
            Content::Unique(_) => "unique".to_string(),
            Content::Hidden(_) => "hidden".to_string(),
            Content::Datasource(_) => "datasource".to_string(),
            Content::Empty(_) => "empty".to_string(),
        }
    }
}

impl Default for Content {
    fn default() -> Self {
        Self::Null(NullContent)
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
            Value::Null => Content::Null(NullContent),
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
                    .map(|(key, value)| (key.to_string(), Content::from(value)))
                    .collect();
                Content::Object(ObjectContent {
                    fields,
                    ..Default::default()
                })
            }
            Value::Number(number_value) => {
                let number_content = if number_value.is_f64() {
                    let value = number_value.as_f64().unwrap();
                    NumberContent::F64(number_content::F64::Range(RangeStep::new(
                        value,
                        value + 1.,
                        1.,
                    )))
                } else if number_value.is_u64() {
                    let value = number_value.as_u64().unwrap();
                    NumberContent::U64(number_content::U64::Range(RangeStep::new(
                        value,
                        value + 1,
                        1,
                    )))
                } else if number_value.is_i64() {
                    let value = number_value.as_i64().unwrap();
                    NumberContent::I64(number_content::I64::Range(RangeStep::new(
                        value,
                        value + 1,
                        1,
                    )))
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
            Self::DateTime(date_time_content) => date_time_content.compile(compiler),
            Self::Number(number_content) => number_content.compile(compiler),
            Self::Array(array_content) => array_content.compile(compiler),
            Self::SameAs(same_as_content) => same_as_content.compile(compiler),
            Self::OneOf(one_of_content) => one_of_content.compile(compiler),
            Self::Series(series_content) => series_content.compile(compiler),
            Self::Unique(unique_content) => unique_content.compile(compiler),
            Self::Hidden(hidden_content) => hidden_content.compile(compiler),
            Self::Null(_) => Ok(Graph::null()),
            Self::Datasource(datasource) => datasource.compile(compiler),
            Self::Empty(_) => Err(anyhow!("unexpected empty object")),
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

impl Hash for Weight {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

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
    use paste::paste;

    lazy_static! {
        pub static ref USER_SCHEMA: Content = schema!({
            "type": "object",
            "skip_when_null": true,
            "_uuid": {
                "type": "string",
                "uuid": {},
                "hidden": true
            },
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
            "skip_when_null_": {
                "optional": true,
                "type": "bool",
                "frequency": 0.5
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
        assert!(USER_SCHEMA.accepts(&USER).is_ok());
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
            let in_: DateTimeContent = serde_json::from_value(json!($($inner)*)).unwrap();
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
    }

    #[test]
    #[should_panic(
        expected = "`string` generator is missing a subtype. Try adding `pattern`, `faker`, `categorical`, `serialized`, `uuid`, `truncated`, or `format`"
    )]
    fn string_missing_subtype() {
        let _schema: Content = schema!({
            "type": "array",
            "length": 1,
            "content": {
                "type": "object",
                "s": {
                    "type": "string"
                }
            }
        });
    }

    #[test]
    #[should_panic(
        expected = "`bool` generator is missing a subtype. Try adding `constant`, or `frequency`"
    )]
    fn bool_missing_subtype() {
        let _schema: Content = schema!({
            "type": "array",
            "length": 1,
            "content": {
                "type": "object",
                "b": {
                    "type": "bool"
                }
            }
        });
    }

    #[test]
    #[should_panic(
        expected = "`number` generator is missing a subtype. Try adding `constant`, `range`, or `id`"
    )]
    fn number_missing_subtype() {
        let _schema: Content = schema!({
            "type": "array",
            "length": 1,
            "content": {
                "type": "object",
                "n": {
                    "type": "number"
                }
            }
        });
    }

    #[test]
    #[should_panic(expected = "`date_time` generator is missing a `format` field")]
    fn date_time_missing_subtype() {
        let _schema: Content = schema!({
            "type": "array",
            "length": 1,
            "content": {
                "type": "object",
                "dt": {
                    "type": "date_time"
                }
            }
        });
    }

    #[test]
    #[should_panic(expected = "`array` generator is missing a `length` and `content` field")]
    fn array_missing_subtype() {
        let _schema: Content = schema!({
            "type": "array",
            "length": 1,
            "content": {
                "type": "object",
                "a": {
                    "type": "array"
                }
            }
        });
    }

    #[test]
    fn object_missing_subtype() {
        let _schema: Content = schema!({
            "type": "array",
            "length": 1,
            "content": {
                "type": "object",
                "o": {
                    "type": "object"
                }
            }
        });
    }

    #[test]
    #[should_panic(expected = "`same_as` generator is missing a `ref` field")]
    fn same_as_missing_subtype() {
        let _schema: Content = schema!({
            "type": "array",
            "length": 1,
            "content": {
                "type": "object",
                "sa": {
                    "type": "same_as"
                }
            }
        });
    }

    #[test]
    #[should_panic(expected = "`one_of` generator is missing a `variants` field")]
    fn one_of_missing_subtype() {
        let _schema: Content = schema!({
            "type": "array",
            "length": 1,
            "content": {
                "type": "object",
                "oo": {
                    "type": "one_of"
                }
            }
        });
    }

    #[test]
    #[should_panic(
        expected = "`series` generator is missing a variant. Try adding `incrementing`, `poisson`, `cyclical`, or `zip`"
    )]
    fn series_missing_subtype() {
        let _schema: Content = schema!({
            "type": "array",
            "length": 1,
            "content": {
                "type": "object",
                "s": {
                    "type": "series"
                }
            }
        });
    }

    #[test]
    #[should_panic(expected = "`unique` generator is missing a `content` field")]
    fn unique_missing_subtype() {
        let _schema: Content = schema!({
            "type": "array",
            "length": 1,
            "content": {
                "type": "object",
                "u": {
                    "type": "unique"
                }
            }
        });
    }

    #[test]
    #[should_panic(expected = "`datasource` generator is missing a `path` field")]
    fn datasource_missing_subtype() {
        let _schema: Content = schema!({
            "type": "array",
            "length": 1,
            "content": {
                "type": "object",
                "d": {
                    "type": "datasource"
                }
            }
        });
    }

    #[test]
    #[should_panic(expected = "`hidden` generator is missing a `content` field")]
    fn hidden_missing_subtype() {
        let _schema: Content = schema!({
            "type": "array",
            "length": 1,
            "content": {
                "type": "object",
                "h": {
                    "type": "hidden"
                }
            }
        });
    }

    #[test]
    fn null_missing_subtype() {
        let _schema: Content = schema!({
            "type": "array",
            "length": 1,
            "content": {
                "type": "object",
                "n": {
                    "type": "null"
                }
            }
        });
    }

    macro_rules! unexpected_content_tests {
        ($($name:ident: {$($schema:tt)*},)*) => {
        $(paste!{
            #[test]
            #[should_panic(expected = "is expected to be a field of")]
            fn [<unexpected_content_ $name>]() {
                let _schema: Content = schema!({$($schema)*});
            }
        })*
        }
    }

    unexpected_content_tests! {
        format: {
            "type": "array",
            "length": 1,
            "content": {
                "type": "object",
                "s": {
                    "type": "string",
                    "format": "Hello {name}",
                    "arguments": {
                        "name": "synth"
                    }
                }
            }
        },
        low: {
            "type": "array",
            "length": {
                "type": "number",
                "low": 1
            },
            "content": {
                "type": "object"
            }
        },
        high: {
            "type": "array",
            "length": {
                "type": "number",
                "high": 10
            },
            "content": {
                "type": "object"
            }
        },
        step: {
            "type": "array",
            "length": {
                "type": "number",
                "step": 2
            },
            "content": {
                "type": "object"
            }
        },
        include_high: {
            "type": "array",
            "length": {
                "type": "number",
                "include_high": true
            },
            "content": {
                "type": "object"
            }
        },
        include_low: {
            "type": "array",
            "length": {
                "type": "number",
                "include_low": true
            },
            "content": {
                "type": "object"
            }
        },
        start_at: {
            "type": "array",
            "length": {
                "type": "number",
                "start_at": 5
            },
            "content": {
                "type": "object"
            }
        },
        start: {
            "type": "array",
            "length": 1,
            "content": {
                "type": "object",
                "s": {
                    "type": "series",
                    "start": "2021-02-01 09:00:00"
                }
            }
        },
        increment: {
            "type": "array",
            "length": 1,
            "content": {
                "type": "object",
                "s": {
                    "type": "series",
                    "increment": "1m"
                }
            }
        },
        rate: {
            "type": "array",
            "length": 1,
            "content": {
                "type": "object",
                "s": {
                    "type": "series",
                    "rate": "1m"
                }
            }
        },
        max_rate: {
            "type": "array",
            "length": 1,
            "content": {
                "type": "object",
                "s": {
                    "type": "series",
                    "max_rate": "1m"
                }
            }
        },
        min_rate: {
            "type": "array",
            "length": 1,
            "content": {
                "type": "object",
                "s": {
                    "type": "series",
                    "min_rate": "1m"
                }
            }
        },
        period: {
            "type": "array",
            "length": 1,
            "content": {
                "type": "object",
                "s": {
                    "type": "series",
                    "period": "1d"
                }
            }
        },
        series: {
            "type": "array",
            "length": 1,
            "content": {
                "type": "object",
                "s": {
                    "type": "series",
                    "series": []
                }
            }
        },
    }
}
