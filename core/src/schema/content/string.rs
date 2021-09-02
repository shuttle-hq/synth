use std::sync::Arc;

use super::prelude::*;
use super::Categorical;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum StringContent {
    Pattern(RegexContent),
    Faker(FakerContent),
    DateTime(DateTimeContent),
    Categorical(Categorical<String>),
    Serialized(SerializedContent),
    Uuid(Uuid),
    Truncated(TruncatedContent),
    Format(FormatContent),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Uuid;

impl StringContent {
    pub fn kind(&self) -> &str {
        match self {
            Self::Pattern(_) => "pattern",
            Self::Faker(faker) => faker.kind(),
            Self::DateTime(date_time) => date_time.kind(),
            Self::Categorical(_) => "categorical",
            Self::Serialized(_) => "serialized",
            Self::Uuid(_) => "uuid",
            Self::Truncated(_) => "truncated",
            Self::Format(_) => "format",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegexContent(String, RandRegex);

impl Display for RegexContent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq for RegexContent {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl RegexContent {
    pub fn to_regex(&self) -> RandRegex {
        self.1.clone()
    }

    pub fn pattern(pattern: String) -> Result<Self, RegexError> {
        Self::compile(pattern, 1)
    }

    pub fn compile(pattern: String, max_repeat: u32) -> Result<Self, RegexError> {
        let rand_regex = RandRegex::compile(pattern.as_str(), max_repeat)?;
        Ok(Self(pattern, rand_regex))
    }
}

impl<'de> Deserialize<'de> for RegexContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct RegexVisitor;
        impl<'de> Visitor<'de> for RegexVisitor {
            type Value = RegexContent;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "a string")
            }
            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(s.to_string())
            }
            fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let rand_regex = RandRegex::compile(s.as_str(), 32).map_err(|e| {
                    let msg = format!("bad regex: {}", e);
                    E::custom(msg)
                })?;
                Ok(RegexContent(s, rand_regex))
            }
        }
        deserializer.deserialize_string(RegexVisitor)
    }
}

impl Serialize for RegexContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.as_str())
    }
}

impl Default for RegexContent {
    fn default() -> Self {
        let pattern = "[a-zA-Z0-9]*".to_string();
        RegexContent::compile(pattern, 32).unwrap()
    }
}

impl Default for StringContent {
    fn default() -> Self {
        Self::Pattern(RegexContent::default())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FakerContentArgument(Value);

impl FakerContentArgument {
    pub fn as_inner(&self) -> &Value {
        &self.0
    }
}

impl<'de> Deserialize<'de> for FakerContentArgument {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match &value {
            Value::Number(_) |
            Value::String(_) |
            Value::Bool(_) => Ok(Self(value)),
            _ => {
                Err(D::Error::custom("invalid argument for a faker generator: can only be of a primitive type (i.e. one of string, number or boolean)"))
            }
        }
    }
}

impl Serialize for FakerContentArgument {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FakerContent {
    pub generator: String,
    /// deprecated: Use FakerArgs::locale instead
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub locales: Vec<String>,
    #[serde(flatten)]
    pub args: crate::graph::string::FakerArgs,
}

impl FakerContent {
    fn kind(&self) -> &str {
        self.generator.as_ref()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "serializer")]
pub enum SerializedContent {
    Json(JsonContent),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub struct TruncatedContent {
    content: Box<Content>,
    length: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub struct FormatContent {
    format: String,
    pub arguments: HashMap<String, Content>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct JsonContent {
    content: Box<Content>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Hash, Serialize)]
pub struct ChronoValueAndFormat {
    pub value: ChronoValue,
    pub format: Arc<str>,
}

impl ChronoValueAndFormat {
    pub fn format_to_string(&self) -> String {
        match self.value {
            ChronoValue::NaiveDate(d) => d.format(&self.format),
            ChronoValue::NaiveTime(t) => t.format(&self.format),
            ChronoValue::NaiveDateTime(dt) => dt.format(&self.format),
            ChronoValue::DateTime(dt) => dt.format(&self.format),
        }.to_string()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Hash, Serialize)]
pub enum ChronoValue {
    NaiveDate(NaiveDate),
    NaiveTime(NaiveTime),
    NaiveDateTime(NaiveDateTime),
    DateTime(DateTime<FixedOffset>),
}

impl std::ops::Add<Duration> for ChronoValue {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        match self {
            Self::NaiveDate(n_d) => Self::NaiveDate(n_d + rhs),
            Self::NaiveTime(n_t) => Self::NaiveTime(n_t + rhs),
            Self::NaiveDateTime(n_dt) => Self::NaiveDateTime(n_dt + rhs),
            Self::DateTime(dt) => Self::DateTime(dt + rhs),
        }
    }
}

impl std::ops::Add<StdDuration> for ChronoValue {
    type Output = Self;

    fn add(self, rhs: StdDuration) -> Self::Output {
        // @brokad: this may blow up in some edge cases
        self.add(Duration::from_std(rhs).unwrap())
    }
}

impl std::fmt::Display for ChronoValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NaiveDate => write!(f, "naive date"),
            Self::NaiveTime => write!(f, "naive time"),
            Self::NaiveDateTime => write!(f, "naive date time"),
            Self::DateTime => write!(f, "date time"),
        }
    }
}

impl ChronoValue {
    pub fn common_variant(&self, other: &Self) -> Option<ChronoValueType> {
        if self.type_() == other.type_() {
            Some(self.type_())
        } else {
            None
        }
    }

    pub fn delta_to(&self, other: &Self) -> Option<StdDuration> {
        let res = match (self, other) {
            (Self::NaiveDate(left), Self::NaiveDate(right)) => Some(*right - *left),
            (Self::NaiveTime(left), Self::NaiveTime(right)) => Some(*right - *left),
            (Self::NaiveDateTime(left), Self::NaiveDateTime(right)) => Some(*right - *left),
            (Self::DateTime(left), Self::DateTime(right)) => Some(*right - *left),
            _ => None,
        };
        // @brokad: this may blow up in some edge cases
        res.map(|c_duration| c_duration.to_std().unwrap())
    }

    pub fn type_(&self) -> ChronoValueType {
        match self {
            Self::DateTime(_) => ChronoValueType::DateTime,
            Self::NaiveDateTime(_) => ChronoValueType::NaiveDateTime,
            Self::NaiveTime(_) => ChronoValueType::NaiveTime,
            Self::NaiveDate(_) => ChronoValueType::NaiveDate,
        }
    }

    pub fn now() -> DateTime<FixedOffset> {
        FixedOffset::east(0).from_utc_datetime(&Utc::now().naive_local())
    }

    pub fn origin() -> DateTime<FixedOffset> {
        FixedOffset::east(0).ymd(1970, 1, 1).and_hms(0, 0, 0)
    }

    pub fn default_of(default: DateTime<FixedOffset>, type_: ChronoValueType) -> Self {
        match type_ {
            ChronoValueType::DateTime => Self::DateTime(default),
            ChronoValueType::NaiveDateTime => Self::NaiveDateTime(default.naive_local()),
            ChronoValueType::NaiveTime => Self::NaiveTime(default.time()),
            ChronoValueType::NaiveDate => Self::NaiveDate(default.naive_local().date()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ChronoValueType {
    NaiveDate,
    NaiveTime,
    NaiveDateTime,
    DateTime,
}

impl Default for ChronoValueType {
    fn default() -> Self {
        Self::DateTime
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DateTimeContent {
    pub format: String,
    pub type_: ChronoValueType,
    pub begin: Option<ChronoValue>,
    pub end: Option<ChronoValue>,
}

impl DateTimeContent {
    fn kind(&self) -> &str {
        self.format.as_str()
    }
}

pub mod datetime_content {
    use super::{ChronoValue, ChronoValueType, DateTimeContent, Error};
    use anyhow::Result;
    use serde::{Deserialize, Serialize};

    use chrono::{
        format::{parse as strptime, StrftimeItems},
        DateTime, FixedOffset,
    };

    #[derive(Debug)]
    pub struct ChronoValueFormatter<'a>(&'a str, Option<ChronoValueType>);

    impl<'a> ChronoValueFormatter<'a> {
        pub fn new_with(src: &'a str, hint: Option<ChronoValueType>) -> Self {
            Self(src, hint)
        }

        pub fn new(src: &'a str) -> Self {
            Self::new_with(src, None)
        }

        pub fn parse(&self, content: &str) -> Result<ChronoValue> {
            debug!(
                "parsing a chrono content from string '{}' ({:?})",
                content, self
            );

            let mut parsed = chrono::format::Parsed::new();

            strptime(&mut parsed, content, StrftimeItems::new(self.0)).map_err(|err| {
                failed!(
                    target: Debug,
                    "could not parse '{}' as a chrono content with fmt='{}': {}",
                    content,
                    self.0,
                    err
                )
            })?;

            if let Some(hint) = self.1 {
                match hint {
                    ChronoValueType::DateTime => Ok(ChronoValue::DateTime(parsed.to_datetime()?)),
                    ChronoValueType::NaiveDateTime => Ok(ChronoValue::NaiveDateTime(
                        parsed.to_naive_date()?.and_time(parsed.to_naive_time()?),
                    )),
                    ChronoValueType::NaiveDate => {
                        Ok(ChronoValue::NaiveDate(parsed.to_naive_date()?))
                    }
                    ChronoValueType::NaiveTime => {
                        Ok(ChronoValue::NaiveTime(parsed.to_naive_time()?))
                    }
                }
            } else {
                parsed
                    .to_datetime()
                    .map(ChronoValue::DateTime)
                    .or_else(|err| {
                        debug!("a chrono content failed to parse as a datetime: {}", err);
                        parsed
                            .to_naive_date()
                            .map(|date| match parsed.to_naive_time() {
                                Ok(time) => ChronoValue::NaiveDateTime(date.and_time(time)),
                                Err(_) => ChronoValue::NaiveDate(date),
                            })
                            .or_else(|err| {
                                debug!(
                                    "a chrono content failed to parse as a naive datetime: {}",
                                    err
                                );
                                Ok(ChronoValue::NaiveTime(parsed.to_naive_time()?))
                            })
                    })
            }
        }

        #[allow(dead_code)]
        fn parse_or_default_of(
            &self,
            opt: Option<String>,
            def: DateTime<FixedOffset>,
            hint: ChronoValueType,
        ) -> Result<ChronoValue> {
            match opt.map(|inner| self.parse(&inner)).transpose()? {
                Some(inner) => Ok(inner),
                None => {
                    let default = ChronoValue::default_of(def, hint);
                    let fmt = ChronoValueFormatter::new_with(self.0, None);
                    fmt.parse(&fmt.format(&default)?)
                }
            }
        }

        pub fn format(&self, c: &ChronoValue) -> Result<String, Error> {
            use std::fmt::Write;
            let mut buf = String::new();
            let display = match c {
                ChronoValue::DateTime(dt) => dt.format(self.0),
                ChronoValue::NaiveDateTime(n_dt) => n_dt.format(self.0),
                ChronoValue::NaiveTime(n_t) => n_t.format(self.0),
                ChronoValue::NaiveDate(n_d) => n_d.format(self.0),
            };
            buf.write_fmt(format_args!("{}", display)).map_err(|err| {
                failed_crate!(
                    target: Release,
                    "could not format date time of type '{}' with '{}': {}",
                    c.type_(),
                    &self.0,
                    err
                )
            })?;
            buf.shrink_to_fit();
            Ok(buf)
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub(super) struct SerdeDateTimeContent {
        format: String,
        #[serde(rename = "subtype")]
        type_: Option<ChronoValueType>,
        begin: Option<String>,
        end: Option<String>,
    }

    impl SerdeDateTimeContent {
        pub(super) fn into_datetime_content(self) -> Result<DateTimeContent> {
            debug!("interpreting a shadow datetime content {:?}", self);

            let src = &self.format;
            let fmt = ChronoValueFormatter::new_with(src, self.type_);

            let type_ = self.type_.unwrap_or_default();
            let begin = self
                .begin
                .map(|begin| fmt.parse(begin.as_str()))
                .transpose()?;
            let end = self.end.map(|end| fmt.parse(end.as_str())).transpose()?;

            if let (Some(begin), Some(end)) = (begin.as_ref(), end.as_ref()) {
                if begin > end {
                    return Err(failed!(
                        target: Release,
                        "begin is after end: begin={}, end={}",
                        fmt.format(begin).unwrap(), // should be alright exactly at this point
                        fmt.format(end).unwrap()
                    ));
                }
            }

            let common_variant = begin
                .as_ref()
                .and_then(|begin| begin.common_variant(end.as_ref()?));

            match common_variant {
                Some(variant) if variant != type_ => Err(failed!(target: Release, "content types of 'begin' and 'end' mismatch: begin is a {:?}, end is a {:?}; this is not allowed here. Try specifying the 'type' field.", begin, end)),
                _ => Ok(DateTimeContent {
                    format: self.format,
                    type_,
                    begin,
                    end,
                })
            }
        }

        pub(super) fn from_datetime_content(c: &DateTimeContent) -> Result<Self> {
            let fmt = ChronoValueFormatter::new_with(&c.format, None);
            Ok(Self {
                format: c.format.to_string(),
                type_: Some(c.type_),
                begin: c
                    .begin
                    .as_ref()
                    .map(|begin| fmt.format(begin))
                    .transpose()?,
                end: c.end.as_ref().map(|end| fmt.format(end)).transpose()?,
            })
        }
    }
}

use crate::graph::string::Serialized;
pub use datetime_content::ChronoValueFormatter;
use std::ops::Add;

impl Serialize for DateTimeContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        datetime_content::SerdeDateTimeContent::from_datetime_content(self)
            .map_err(S::Error::custom)
            .and_then(|content| content.serialize(serializer))
    }
}

impl<'de> Deserialize<'de> for DateTimeContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        datetime_content::SerdeDateTimeContent::deserialize(deserializer)
            .and_then(|inter| inter.into_datetime_content().map_err(D::Error::custom))
    }
}

impl Compile for StringContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, mut compiler: C) -> Result<Graph> {
        let string_node = match self {
            StringContent::Format(FormatContent { format, arguments }) => {
                let args = FormatArgs {
                    named: arguments
                        .iter()
                        .map(|(name, value)| Ok((name.to_string(), compiler.build(name, value)?)))
                        .collect::<Result<_>>()?,
                    ..Default::default()
                };
                RandomString::from(Format::new(format.to_string(), args)).into()
            }
            StringContent::Pattern(pattern) => RandomString::from(pattern.to_regex()).into(),
            StringContent::Faker(FakerContent {
                generator,
                args,
                locales: _, // to combine locales from the 'locales' field and the args::locales,
                            // we should impl Hash on locale and then put them in a Set
            }) => RandomString::from(RandFaker::new(generator.clone(), args.clone())?).into(),
            StringContent::DateTime(DateTimeContent {
                begin,
                end,
                format,
                type_,
            }) => {
                let begin = begin
                    .clone()
                    .unwrap_or_else(|| ChronoValue::default_of(ChronoValue::now(), *type_));
                let end = end
                    .clone()
                    .unwrap_or_else(|| ChronoValue::default_of(ChronoValue::now(), *type_));
                RandomDateTime::new(begin..end, format).into()
            }
            StringContent::Categorical(cat) => RandomString::from(cat.clone()).into(),
            StringContent::Serialized(sc) => match sc {
                SerializedContent::Json(serialized_json_content) => {
                    let inner = serialized_json_content.content.compile(compiler)?;
                    RandomString::from(Serialized::new_json(inner)).into()
                }
            },
            StringContent::Truncated(trunc) => {
                let inner = trunc.content.compile(compiler)?;
                RandomString::from(Truncated::new(trunc.length, inner)?).into()
            }
            StringContent::Uuid(_uuid) => RandomString::from(UuidGen {}).into(),
        };
        Ok(Graph::String(string_node))
    }
}
