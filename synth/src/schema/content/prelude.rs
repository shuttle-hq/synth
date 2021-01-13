// std-dtolnay
pub(super) use anyhow::{Context, Result};
pub(super) use serde::{
    de::{Error as DeError, Visitor},
    ser::Error as SeError,
    Deserialize, Serialize,
};
pub(super) use serde_json::{Map, Number, Value};
pub(super) type JsonObject = Map<String, Value>;

// std
pub(super) use std::cell::RefCell;
pub(super) use std::collections::HashMap;
pub(super) use std::convert::TryFrom;
pub(super) use std::fmt::{Display, Formatter};
pub(super) use std::iter::{FromIterator, Peekable};
pub(super) use std::rc::Rc;
pub(super) use std::time::Duration as StdDuration;

// other
pub(super) use chrono::{
    DateTime, Duration, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc,
};
pub(super) use num::{Bounded, One};
pub(super) use pyo3::{conversion::ToPyObject, PyObject, Python};
pub(super) use rand::distributions::Bernoulli;
pub(super) use rand_regex::{Error as RegexError, Regex as RandRegex};

// bynar
pub(super) use synth_generator::{prelude::*, Chain, OneOf, Seed};

// crate
pub(super) use super::{suggest_closest, Content, Find};
pub(super) use crate::error::Error;
pub(super) use crate::gen::{
    BoolModel, Compile, Compiler, FakerSeed, Model, NumberModel, PrimitiveModel, RandDateTime,
    StringModel,
};
pub(super) use crate::schema::{MergeStrategy, OptionalMergeStrategy};
