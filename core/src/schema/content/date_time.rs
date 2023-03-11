#![allow(clippy::derive_partial_eq_without_eq)]
use super::prelude::*;

use chrono::format::{parse as strptime, StrftimeItems};

use std::sync::Arc;

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
        }
        .to_string()
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
        FixedOffset::east_opt(0).unwrap().from_utc_datetime(&Utc::now().naive_local())
    }

    pub fn origin() -> DateTime<FixedOffset> {
        FixedOffset::east_opt(0).unwrap().with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap()
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Hash)]
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

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct DateTimeContent {
    pub format: String,
    pub type_: ChronoValueType,
    pub begin: Option<ChronoValue>,
    pub end: Option<ChronoValue>,
}

#[derive(Debug)]
pub struct ChronoValueFormatter<'a>(&'a str);

impl<'a> ChronoValueFormatter<'a> {
    pub fn new(src: &'a str) -> Self {
        Self(src)
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

    pub fn format(&self, c: &ChronoValue) -> Result<String, Error> {
        use std::fmt::Write;
        let mut buf = String::new();
        let display = match c {
            ChronoValue::DateTime(dt) => dt.format(self.0),
            ChronoValue::NaiveDateTime(n_dt) => n_dt.format(self.0),
            ChronoValue::NaiveTime(n_t) => n_t.format(self.0),
            ChronoValue::NaiveDate(n_d) => n_d.format(self.0),
        };
        buf.write_fmt(format_args!("{display}")).map_err(|err| {
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
struct SerdeDateTimeContent {
    format: String,
    #[serde(rename = "subtype")]
    type_: Option<ChronoValueType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    begin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<String>,
}

impl SerdeDateTimeContent {
    fn into_datetime_content(self) -> Result<DateTimeContent> {
        debug!("interpreting a shadow datetime content {:?}", self);

        let fmt = ChronoValueFormatter::new(&self.format);

        let begin = self
            .begin
            .map(|begin| fmt.parse(begin.as_str()))
            .transpose()?;

        let end = self.end.map(|end| fmt.parse(end.as_str())).transpose()?;

        Ok(DateTimeContent {
            format: self.format,
            type_: infer_date_time_type(self.type_, &begin, &end)?,
            begin,
            end,
        })
    }

    fn from_datetime_content(c: &DateTimeContent) -> Result<Self> {
        let fmt = ChronoValueFormatter::new(&c.format);
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

impl Serialize for DateTimeContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerdeDateTimeContent::from_datetime_content(self)
            .map_err(S::Error::custom)
            .and_then(|content| content.serialize(serializer))
    }
}

impl<'de> Deserialize<'de> for DateTimeContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        SerdeDateTimeContent::deserialize(deserializer)
            .and_then(|inter| inter.into_datetime_content().map_err(D::Error::custom))
    }
}

impl Compile for DateTimeContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, _compiler: C) -> Result<Graph> {
        let begin = self
            .begin
            .clone()
            .unwrap_or_else(|| ChronoValue::default_of(ChronoValue::now(), self.type_));
        let end = self
            .end
            .clone()
            .unwrap_or_else(|| ChronoValue::default_of(ChronoValue::now(), self.type_));
        if begin > end {
            let fmt = ChronoValueFormatter::new(&self.format);
            return Err(anyhow!(
                "begin is after end: begin={}, end={}",
                fmt.format(&begin).unwrap(),
                fmt.format(&end).unwrap()
            ));
        }
        let date_time_node = RandomDateTime::new(begin..end, &self.format).into();
        Ok(Graph::DateTime(date_time_node))
    }
}

fn infer_date_time_type(
    specified_type: Option<ChronoValueType>,
    begin: &Option<ChronoValue>,
    end: &Option<ChronoValue>,
) -> Result<ChronoValueType> {
    let inferred_type = match (
        begin.as_ref().map(ChronoValue::type_),
        end.as_ref().map(ChronoValue::type_),
    ) {
        (Some(b), Some(e)) => {
            if b == e {
                Some(b)
            } else {
                return Err(anyhow!(
                    "The date/time types of 'begin' and 'end' do not match"
                ));
            }
        }
        (Some(t), None) | (None, Some(t)) => Some(t),
        (None, None) => None,
    };

    match (inferred_type, specified_type) {
        (Some(inferred), Some(specified)) => {
            if inferred == specified {
                Ok(inferred)
            } else {
                Err(anyhow!("The date/time type specified with 'subtype' does align with the type inferred from 'begin'/'end'"))
            }
        }
        (Some(t), None) | (None, Some(t)) => Ok(t),
        (None, None) => Ok(ChronoValueType::default()),
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::compile::NamespaceCompiler;
    use chrono::naive::{MAX_DATE, MIN_DATE};

    macro_rules! date_time_bounds_test_ok {
        ($begin:expr, $end:expr) => {
            let unspecified_begin_end = DateTimeContent {
                format: "yyyy-MM-dd".to_string(),
                type_: ChronoValueType::NaiveDate,
                begin: $begin,
                end: $end,
            };

            let content = Content::DateTime(unspecified_begin_end);

            let compiler = NamespaceCompiler::new_flat(&content);

            assert!(compiler.compile().is_ok());
        };
    }

    macro_rules! date_time_bounds_test_err {
        ($begin:expr, $end:expr) => {
            let unspecified_begin_end = DateTimeContent {
                format: "yyyy-MM-dd".to_string(),
                type_: ChronoValueType::NaiveDate,
                begin: $begin,
                end: $end,
            };

            let content = Content::DateTime(unspecified_begin_end);

            let compiler = NamespaceCompiler::new_flat(&content);

            assert!(compiler.compile().is_err());
        };
    }

    #[test]
    fn date_time_compile() {
        date_time_bounds_test_ok!(None, None);
        date_time_bounds_test_ok!(None, Some(ChronoValue::NaiveDate(MAX_DATE)));
        date_time_bounds_test_ok!(Some(ChronoValue::NaiveDate(MIN_DATE)), None);
        date_time_bounds_test_ok!(
            Some(ChronoValue::NaiveDate(MIN_DATE)),
            Some(ChronoValue::NaiveDate(MAX_DATE))
        );

        date_time_bounds_test_err!(Some(ChronoValue::NaiveDate(MAX_DATE)), None);
        date_time_bounds_test_err!(None, Some(ChronoValue::NaiveDate(MIN_DATE)));
    }

    #[test]
    fn date_time_subtype_inference() {
        let some_time = Some(ChronoValue::NaiveTime(NaiveTime::from_hms(1, 2, 3)));
        let some_date = Some(ChronoValue::NaiveDate(NaiveDate::from_ymd(2000, 1, 1)));

        assert_eq!(
            infer_date_time_type(Some(ChronoValueType::DateTime), &None, &None).unwrap(),
            ChronoValueType::DateTime
        );
        assert_eq!(
            infer_date_time_type(Some(ChronoValueType::NaiveTime), &some_time, &some_time).unwrap(),
            ChronoValueType::NaiveTime
        );
        assert_eq!(
            infer_date_time_type(Some(ChronoValueType::NaiveTime), &some_time, &None).unwrap(),
            ChronoValueType::NaiveTime
        );
        assert_eq!(
            infer_date_time_type(None, &some_time, &None).unwrap(),
            ChronoValueType::NaiveTime
        );
        assert_eq!(
            infer_date_time_type(None, &None, &None).unwrap(),
            ChronoValueType::default()
        );
        assert!(infer_date_time_type(Some(ChronoValueType::NaiveDate), &None, &some_time).is_err());
        assert!(infer_date_time_type(Some(ChronoValueType::NaiveTime), &None, &some_date).is_err());
        assert!(infer_date_time_type(None, &some_time, &some_date).is_err());
    }
}
