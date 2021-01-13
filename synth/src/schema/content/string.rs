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
}

impl StringContent {
    pub fn kind(&self) -> &str {
        match self {
            Self::Pattern(_) => "pattern",
            Self::Faker(faker) => faker.kind(),
            Self::DateTime(date_time) => date_time.kind(),
            Self::Categorical(_) => "categorical",
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
    #[serde(flatten)]
    pub args: HashMap<String, FakerContentArgument>,
}

impl FakerContent {
    fn kind(&self) -> &str {
        self.generator.as_ref()
    }
}

impl ToPyObject for FakerContentArgument {
    fn to_object(&self, py: Python) -> PyObject {
        match &self.0 {
            Value::Bool(x) => x.to_object(py),
            Value::String(x) => x.to_object(py),
            Value::Number(number) => {
                number
                    .as_u64()
                    .map(|n| n.to_object(py))
                    .or_else(|| number.as_i64().map(|n| n.to_object(py)))
                    .or_else(|| number.as_f64().map(|n| n.to_object(py)))
                    .unwrap() // serde_json::Number API contract
            }
            _ => unreachable!(), // would not be constructed
        }
    }
}

impl Compile for FakerContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, compiler: C) -> Result<Model> {
        let seed = FakerSeed {
            generator: self.generator.clone(),
            python: compiler.python()?,
            args: self.args.clone(),
        };
        Ok(Model::Primitive(PrimitiveModel::Faker(seed.once())))
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum ChronoContent {
    NaiveDate(NaiveDate),
    NaiveTime(NaiveTime),
    NaiveDateTime(NaiveDateTime),
    DateTime(DateTime<FixedOffset>),
}

impl std::ops::Add<Duration> for ChronoContent {
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

impl std::ops::Add<StdDuration> for ChronoContent {
    type Output = Self;

    fn add(self, rhs: StdDuration) -> Self::Output {
        // @brokad: this may blow up in some edge cases
        self.add(Duration::from_std(rhs).unwrap())
    }
}

impl std::fmt::Display for ChronoContent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NaiveDate(_) => write!(f, "naive date"),
            Self::NaiveTime(_) => write!(f, "naive time"),
            Self::NaiveDateTime(_) => write!(f, "naive date time"),
            Self::DateTime(_) => write!(f, "date time"),
        }
    }
}

impl ChronoContent {
    pub fn common_variant(&self, other: &Self) -> Option<ChronoContentType> {
        if self.to_string() == other.to_string() {
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

    pub fn type_(&self) -> ChronoContentType {
        match self {
            Self::DateTime(_) => ChronoContentType::DateTime,
            Self::NaiveDateTime(_) => ChronoContentType::NaiveDateTime,
            Self::NaiveTime(_) => ChronoContentType::NaiveTime,
            Self::NaiveDate(_) => ChronoContentType::NaiveDate,
        }
    }

    pub fn now() -> DateTime<FixedOffset> {
        FixedOffset::east(0).from_utc_datetime(&Utc::now().naive_local())
    }

    pub fn origin() -> DateTime<FixedOffset> {
        FixedOffset::east(0).ymd(1970, 1, 1).and_hms(0, 0, 0)
    }

    pub fn default_of(default: DateTime<FixedOffset>, type_: ChronoContentType) -> Self {
        match type_ {
            ChronoContentType::DateTime => Self::DateTime(default),
            ChronoContentType::NaiveDateTime => Self::NaiveDateTime(default.naive_local()),
            ChronoContentType::NaiveTime => Self::NaiveTime(default.time()),
            ChronoContentType::NaiveDate => Self::NaiveDate(default.naive_local().date()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ChronoContentType {
    NaiveDate,
    NaiveTime,
    NaiveDateTime,
    DateTime,
}

impl Default for ChronoContentType {
    fn default() -> Self {
        Self::DateTime
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DateTimeContent {
    pub format: String,
    pub type_: ChronoContentType,
    pub begin: Option<ChronoContent>,
    pub end: Option<ChronoContent>,
}

impl DateTimeContent {
    fn kind(&self) -> &str {
        self.format.as_str()
    }
}

impl Compile for DateTimeContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, _compiler: C) -> Result<Model> {
        let begin = self.begin.clone().unwrap_or(ChronoContent::default_of(
            ChronoContent::origin(),
            self.type_,
        ));
        let end = self
            .end
            .clone()
            .unwrap_or(begin.clone() + Duration::weeks(52));
        let rand_datetime = RandDateTime::new(begin..end, &self.format);
        let model = Seed::new_with(rand_datetime).once();
        Ok(Model::Primitive(PrimitiveModel::String(
            StringModel::Chrono(model),
        )))
    }
}

pub mod datetime_content {
    use super::{ChronoContent, ChronoContentType, DateTimeContent};
    use anyhow::Result;
    use serde::{Deserialize, Serialize};

    use chrono::{
        format::{parse as strptime, StrftimeItems},
        DateTime, FixedOffset,
    };

    #[derive(Debug)]
    pub struct ChronoContentFormatter<'a>(&'a str, Option<ChronoContentType>);

    impl<'a> ChronoContentFormatter<'a> {
        pub fn new_with(src: &'a str, hint: Option<ChronoContentType>) -> Self {
            Self(src, hint)
        }

        pub fn new(src: &'a str) -> Self {
            Self::new_with(src, None)
        }

        pub fn parse(&self, content: &str) -> Result<ChronoContent> {
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
                    ChronoContentType::DateTime => {
                        Ok(ChronoContent::DateTime(parsed.to_datetime()?))
                    }
                    ChronoContentType::NaiveDateTime => Ok(ChronoContent::NaiveDateTime(
                        parsed.to_naive_date()?.and_time(parsed.to_naive_time()?),
                    )),
                    ChronoContentType::NaiveDate => {
                        Ok(ChronoContent::NaiveDate(parsed.to_naive_date()?))
                    }
                    ChronoContentType::NaiveTime => {
                        Ok(ChronoContent::NaiveTime(parsed.to_naive_time()?))
                    }
                }
            } else {
                parsed
                    .to_datetime()
                    .map(|dt| ChronoContent::DateTime(dt))
                    .or_else(|err| {
                        debug!("a chrono content failed to parse as a datetime: {}", err);
                        parsed
                            .to_naive_date()
                            .and_then(|date| match parsed.to_naive_time() {
                                Ok(time) => Ok(ChronoContent::NaiveDateTime(date.and_time(time))),
                                Err(_) => Ok(ChronoContent::NaiveDate(date)),
                            })
                            .or_else(|err| {
                                debug!(
                                    "a chrono content failed to parse as a naive datetime: {}",
                                    err
                                );
                                Ok(ChronoContent::NaiveTime(parsed.to_naive_time()?))
                            })
                    })
            }
        }

        #[allow(dead_code)]
        fn parse_or_default_of(
            &self,
            opt: Option<String>,
            def: DateTime<FixedOffset>,
            hint: ChronoContentType,
        ) -> Result<ChronoContent> {
            match opt.map(|inner| self.parse(&inner)).transpose()? {
                Some(inner) => Ok(inner),
                None => {
                    let default = ChronoContent::default_of(def, hint);
                    let fmt = ChronoContentFormatter::new_with(self.0, None);
                    fmt.parse(&fmt.format(&default)?)
                }
            }
        }

        pub fn format(&self, c: &ChronoContent) -> Result<String> {
            use std::fmt::Write;
            let mut buf = String::new();
            let display = match c {
                ChronoContent::DateTime(dt) => dt.format(self.0),
                ChronoContent::NaiveDateTime(n_dt) => n_dt.format(self.0),
                ChronoContent::NaiveTime(n_t) => n_t.format(self.0),
                ChronoContent::NaiveDate(n_d) => n_d.format(self.0),
            };
            buf.write_fmt(format_args!("{}", display)).map_err(|err| {
                failed!(
                    target: Debug,
                    "could not format {} with {}: {}",
                    c,
                    self.0,
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
        type_: Option<ChronoContentType>,
        begin: Option<String>,
        end: Option<String>,
    }

    impl SerdeDateTimeContent {
        pub(super) fn to_datetime_content(self) -> Result<DateTimeContent> {
            debug!("interpreting a shadow datetime content {:?}", self);

            let src = &self.format;
            let fmt = ChronoContentFormatter::new_with(src, self.type_);

            let type_ = self.type_.unwrap_or_default();
            let begin = self
                .begin
                .map(|begin| fmt.parse(begin.as_str()))
                .transpose()?;
            let end = self.end.map(|end| fmt.parse(end.as_str())).transpose()?;

            match (begin.as_ref(), end.as_ref()) {
                (Some(begin), Some(end)) => {
                    if begin > end {
                        return Err(failed!(
                            target: Release,
                            "begin is after end: begin={}, end={}",
                            fmt.format(&begin).unwrap(), // should be alright exactly at this point
                            fmt.format(&end).unwrap()
                        ));
                    }
                }
                _ => {}
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
            let fmt = ChronoContentFormatter::new_with(&c.format, None);
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

pub use datetime_content::ChronoContentFormatter;

impl Serialize for DateTimeContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        datetime_content::SerdeDateTimeContent::from_datetime_content(self)
            .map_err(|err| S::Error::custom(err))
            .and_then(|content| content.serialize(serializer))
    }
}

impl<'de> Deserialize<'de> for DateTimeContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        datetime_content::SerdeDateTimeContent::deserialize(deserializer).and_then(|inter| {
            inter
                .to_datetime_content()
                .map_err(|err| D::Error::custom(err))
        })
    }
}

impl Compile for StringContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, compiler: C) -> Result<Model> {
        match self {
            StringContent::Pattern(regex_content) => {
                let gen = Seed::new_with(regex_content.to_regex()).once().into_token();
                Ok(Model::Primitive(PrimitiveModel::String(
                    StringModel::Regex(gen),
                )))
            }
            StringContent::Faker(faker_content) => faker_content.compile(compiler),
            StringContent::DateTime(datetime_content) => datetime_content.compile(compiler),
            StringContent::Categorical(categorical_content) => {
                let gen = Seed::new_with(categorical_content.clone().into())
                    .once()
                    .into_token();
                Ok(Model::Primitive(PrimitiveModel::String(
                    StringModel::Categorical(gen),
                )))
            }
        }
    }
}
