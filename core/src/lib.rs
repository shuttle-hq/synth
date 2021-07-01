#![feature(
    format_args_capture,
    async_closure,
    map_first_last,
    box_patterns,
    error_iter,
    try_blocks,
    min_specialization
)]
#![allow(type_alias_bounds)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate lazy_static;

#[allow(unused_imports)]
#[macro_use]
extern crate serde_json;

extern crate humantime_serde;

#[macro_use]
pub mod error;
pub use error::Error;

#[macro_use]
pub mod schema;
use koto_runtime::value::IndexRange;
use schema::BoolContent;
pub use schema::{Content, Name, Namespace};

pub mod graph;
pub use graph::Graph;

pub mod compile;
pub use compile::{Compile, Compiler};

#[test]
fn create_koto_config() {
    fn _inner() -> Result<(), Box<dyn std::error::Error>> {
        let mut koto = koto::Koto::default();
        bindlang_init(&mut koto.prelude());
        let script = r#"
import synth.Namespace

Namespace({ })"#; // for now leave the namespace empty
        koto.compile(script)?;
        let result_value = koto.run()?;
        let namespace: Namespace = <Namespace as lang_bindings::FromValue>::from_value(
            &lang_bindings::KeyPath::Index(0, None),
            &result_value,
        )?;
        assert_eq!(namespace, Namespace::default());
        Ok(())
    }
    _inner().unwrap()
}

use lang_bindings::{CustomFromValue, FromValue, KeyPath};

impl CustomFromValue for Namespace {
    fn opt_from_value(value: &koto::runtime::Value) -> Option<Self> {
        if let koto::runtime::Value::Map(map) = value {
            return Some(Namespace {
                collections: map
                    .contents()
                    .data
                    .iter()
                    .map(|(key, value)| {
                        let name = if let koto::runtime::Value::Str(s) = key.value() {
                            s.as_str()
                        } else {
                            return None;
                        };
                        Some((
                            <Name as std::str::FromStr>::from_str(name).ok()?,
                            Content::from_value(&KeyPath::Index(0, None), value).ok()?,
                        ))
                    })
                    .collect::<Option<_>>()?,
            });
        }
        None
    }
}

impl CustomFromValue for Content {
    fn opt_from_value(value: &koto::runtime::Value) -> Option<Self> {
        match value {
            koto::runtime::Value::Bool(b) => Some(Content::Bool(BoolContent::Constant(*b))),
            koto::runtime::Value::Number(koto::runtime::ValueNumber::I64(i)) => {
                Some(Content::Number(schema::NumberContent::I64(
                    schema::number_content::I64::Constant(*i),
                )))
            }
            koto::runtime::Value::Number(koto::runtime::ValueNumber::F64(f)) => {
                Some(Content::Number(schema::NumberContent::F64(
                    schema::number_content::F64::Constant(*f),
                )))
            }
            koto::runtime::Value::Range(koto::runtime::IntRange { start, end }) => {
                Some(Content::Number(schema::NumberContent::I64(
                    schema::number_content::I64::Range(schema::content::RangeStep {
                        low: (*start) as i64,
                        high: (*end) as i64,
                        step: 1, // alas, no good way to do anything else yet
                    }),
                )))
            }
            koto::runtime::Value::IndexRange(IndexRange {
                start,
                end: Some(high),
            }) => {
                Some(Content::Number(schema::NumberContent::U64(
                    schema::number_content::U64::Range(schema::content::RangeStep {
                        low: (*start) as u64,
                        high: (*high) as u64,
                        step: 1, // alas, no good way to do anything else yet
                    }),
                )))
            }
            //TODO: More variants
            _ => None,
        }
    }
}

#[test]
fn test_namespace_from_value() {
    let mut koto = koto::Koto::default();
    koto.compile(r#"{"test": true}"#).unwrap();
    let result_value = koto.run().unwrap();
    let namespace: Namespace = <Namespace as lang_bindings::FromValue>::from_value(
        &lang_bindings::KeyPath::Index(0, None),
        &result_value,
    )
    .unwrap();
    assert_eq!(
        namespace,
        Namespace {
            collections: std::iter::once((
                <Name as std::str::FromStr>::from_str("test").unwrap(),
                Content::Bool(BoolContent::Constant(true))
            ))
            .collect()
        }
    )
}

// this is a stop-gap measure to satisfy the `Display` requirements of koto's
// `ExternalValue` trait by forwarding to `Debug`.
macro_rules! debug_display {
    ($($ty:ty),*) => {
        $(
        impl std::fmt::Display for $ty {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
        )*
    }
}

debug_display!(
    schema::content::ChronoValue,
    schema::content::DateTimeContent,
    schema::content::SameAsContent,
    schema::series::SeriesContent,
    schema::series::SeriesVariant,
    schema::series::Poisson,
    schema::series::Incrementing,
    schema::series::Cyclical,
    schema::series::Zip,
    schema::unique::UniqueAlgorithm,
    schema::content::StringContent,
    schema::content::Uuid,
    schema::unique::UniqueContent,
    schema::content::FormatContent,
    schema::content::SerializedContent,
    schema::content::ContentOrRef,
    schema::content::JsonContent,
    schema::content::TruncatedContent
);

lang_bindings::external_value!(crate::schema::Categorical<bool>);
lang_bindings::external_value!(crate::schema::Categorical<String>);
lang_bindings::external_value!(crate::schema::NumberContent);

bindlang::bindlang_main! {
    use crate::graph::string::{FakerArgs, Locale};
    use crate::schema::{ArrayContent, ChronoValue, ChronoValueType, ContentOrRef, DateTimeContent, FakerContent, FieldContent, FieldRef,
        FormatContent, JsonContent, ObjectContent, OneOfContent, RegexContent, SameAsContent, SerializedContent, StringContent, TruncatedContent, Uuid, VariantContent, Weight, optional, required};
    use crate::schema::unique::{unique, UniqueAlgorithm, UniqueContent};
    use crate::schema::series::{Incrementing, SeriesContent, SeriesVariant, Poisson, Cyclical, Zip};
}

/// Create a language runtime we can use to get a namespace from the configuration
pub fn lang_runtime() -> koto::Koto {
    let koto = koto::Koto::default();
    bindlang_init(&mut koto.prelude());
    koto
}

/// compile and run the contents of the koto file and return the result as a namespace
pub fn compile_namespace(runtime: &mut koto::Koto, code: &str) -> Result<Namespace, Error> {
    if let Err(error) = runtime.compile(code) {
        return Err(Error::cast_error(&error));
    }
    let value = match runtime.run() {
        Ok(value) => value,
        Err(error) => return  Err(Error::cast_error(&error)),
    };
    <Namespace as FromValue>::from_value(
        &lang_bindings::KeyPath::Field(std::borrow::Cow::Borrowed(""), None),
        &value,
    ).map_err(|e| Error::cast_error(&e))
}
