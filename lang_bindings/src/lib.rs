#![feature(min_specialization)]

/// bindings for our koto-derived language
///
/// for now we use koto wholesale, will add custom syntax later
// re-export Value for derive macro
pub use koto::runtime::Value;
use koto::runtime::{runtime_error, RuntimeError, ValueNumber};

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

pub enum KeyPath<'parent> {
    Index(usize, Option<&'parent KeyPath<'parent>>),
    Field(Cow<'static, str>, Option<&'parent KeyPath<'parent>>),
}

fn field(name: &'static str) -> KeyPath<'static> {
    KeyPath::Field(Cow::Borrowed(name), None)
}

impl Display for KeyPath<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            &KeyPath::Index(i, parent) => {
                if let Some(p) = parent {
                    write!(f, "{}@{}", p, i)
                } else {
                    write!(f, "{}", i)
                }
            }
            &KeyPath::Field(ref i, parent) => {
                if let Some(p) = parent {
                    write!(f, "{}.{}", p, i)
                } else {
                    write!(f, "{}", i)
                }
            }
        }
    }
}

pub trait StaticExternalValue {
    fn type_str() -> &'static str;
}

pub trait FromValue: Sized {
    fn from_value(key_path: &KeyPath<'_>, value: &Value) -> Result<Self, RuntimeError>;
}

pub trait CustomFromValue: Sized {
    /// can return `Some(..)` value if overridden. Allows to customize the instantiation
    /// from our runtime, e.g. using standard types instead of custom ones.
    fn opt_from_value(_value: &Value) -> Option<Self>;
}

impl<T> CustomFromValue for T {
    default fn opt_from_value(_value: &Value) -> Option<Self> {
        None
    }
}

pub trait RefFromValue {
    fn ref_from_value<R, F: Fn(&Self) -> R>(
        key_path: &KeyPath<'_>,
        value: &Value,
        f: F,
    ) -> Result<R, RuntimeError>;

    fn ref_mut_from_value<R, F: for<'r> Fn(&'r Self) -> R>(
        key_path: &KeyPath<'_>,
        value: &Value,
        _f: F,
    ) -> Result<R, RuntimeError> {
        runtime_error!(
            "Cannot mutate a primitive value ({}) at {}",
            value.type_as_string(),
            key_path
        )
    }
}

#[cold]
pub fn fn_type_error<T: Sized>(
    fn_name: &str,
    inputs: &str,
    args: &[Value],
) -> Result<T, RuntimeError> {
    let mut types = args.iter().map(|v| v.type_as_string());
    let mut argspec = types.next().unwrap_or_else(String::new);
    for ty in types {
        argspec += ", ";
        argspec += &ty;
    }
    runtime_error!("expected {}({:?}), got {}", fn_name, inputs, argspec,)
}

/// Return an error for a missing item
#[cold]
pub fn missing<T: Sized>(key_path: &KeyPath<'_>) -> Result<T, RuntimeError> {
    runtime_error!("Missing value at {}", key_path)
}

/// Return an error for an item of the wrong type
#[cold]
pub fn wrong_type<T: Sized>(
    ty: &'static str,
    key_path: &KeyPath<'_>,
    value: &Value,
) -> Result<T, RuntimeError> {
    runtime_error!(
        "expected value of type {} at {}, found {}",
        ty,
        key_path,
        value.type_as_string(),
    )
}

#[cold]
pub fn not_external_value<T: Sized>(
    key_path: &KeyPath<'_>,
    value: &Value,
) -> Result<T, RuntimeError> {
    runtime_error!("expected external value at {}, found {}", key_path, value)
}

impl FromValue for () {
    fn from_value(_key_path: &KeyPath<'_>, _value: &Value) -> Result<Self, RuntimeError> {
        Ok(())
    }
}

impl FromValue for bool {
    fn from_value(key_path: &KeyPath<'_>, value: &Value) -> Result<Self, RuntimeError> {
        if let Value::Bool(b) = value {
            Ok(*b)
        } else {
            wrong_type("bool", key_path, value)
        }
    }
}

impl RefFromValue for bool {
    fn ref_from_value<R, F: Fn(&Self) -> R>(
        key_path: &KeyPath<'_>,
        value: &Value,
        f: F,
    ) -> Result<R, RuntimeError> {
        if let Value::Bool(b) = value {
            Ok(f(b))
        } else {
            wrong_type("bool", key_path, value)
        }
    }
}

macro_rules! impl_from_value_num {
    (one $ty:ty, $value:path, $category:expr) => {
        impl FromValue for $ty {
            fn from_value(key_path: &KeyPath<'_>, value: &Value) -> Result<Self, RuntimeError> {
                if let Value::Number($value(i)) = value {
                    Ok((*i) as $ty)
                } else {
                    wrong_type($category, key_path, value)
                }
            }
        }

        impl RefFromValue for $ty {
            fn ref_from_value<R, F: Fn(&Self) -> R>(
                key_path: &KeyPath<'_>,
                value: &Value,
                f: F,
            ) -> Result<R, RuntimeError> {
                if let Value::Number($value(i)) = value {
                    Ok(f(&(*i as $ty)))
                } else {
                    wrong_type("integer", key_path, value)
                }
            }
        }
    };
    ($category:expr, $value:path, $($ty:ty),*) => {
        $(impl_from_value_num!(one $ty, $value, $category);)*
    }
}

impl_from_value_num!(
    "integer",
    ValueNumber::I64,
    u8,
    u16,
    u32,
    u64,
    usize,
    i8,
    i16,
    i32,
    i64,
    isize
);
impl_from_value_num!("float", ValueNumber::F64, f32, f64);

impl FromValue for String {
    fn from_value(key_path: &KeyPath<'_>, value: &Value) -> Result<Self, RuntimeError> {
        if let Value::Str(s) = value {
            Ok(s.as_str().to_owned())
        } else {
            wrong_type("string", key_path, value)
        }
    }
}

impl RefFromValue for str {
    fn ref_from_value<R, F: Fn(&Self) -> R>(
        key_path: &KeyPath<'_>,
        value: &Value,
        f: F,
    ) -> Result<R, RuntimeError> {
        if let Value::Str(s) = value {
            Ok(f(s.as_str()))
        } else {
            wrong_type("string", key_path, value)
        }
    }
}

impl<T: FromValue> FromValue for Vec<T> {
    fn from_value(key_path: &KeyPath<'_>, value: &Value) -> Result<Self, RuntimeError> {
        if let Value::List(elems) = value {
            elems
                .data()
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let item = T::from_value(&KeyPath::Index(i, Some(key_path)), v);
                    item
                })
                .collect()
        } else {
            wrong_type("list of items", key_path, value)
        }
    }
}

impl<K: FromValue + PartialOrd + Ord, V: FromValue> FromValue for BTreeMap<K, V> {
    fn from_value(key_path: &KeyPath<'_>, value: &Value) -> Result<Self, RuntimeError> {
        if let Value::Map(map) = value {
            map.contents()
                .data
                .iter()
                .map(|(k, v)| {
                    let kval = k.value();
                    Ok((
                        K::from_value(key_path, kval)?,
                        V::from_value(&KeyPath::Field(kval.to_string().into(), Some(key_path)), v)?,
                    ))
                })
                .collect()
        } else {
            wrong_type("map", key_path, value)
        }
    }
}

impl<T: FromValue> FromValue for Box<T> {
    fn from_value(key_path: &KeyPath<'_>, value: &Value) -> Result<Self, RuntimeError> {
        T::from_value(key_path, value).map(Box::new)
    }
}

impl<T: FromValue, E: Sized> FromValue for Result<T, E> {
    fn from_value(key_path: &KeyPath<'_>, value: &Value) -> Result<Self, RuntimeError> {
        T::from_value(&KeyPath::Field(Cow::Borrowed("ok"), Some(key_path)), value).map(Ok)
    }
}

impl<T: FromValue> FromValue for Option<T> {
    fn from_value(key_path: &KeyPath<'_>, value: &Value) -> Result<Self, RuntimeError> {
        if let &Value::Empty = value {
            Ok(None)
        } else {
            T::from_value(key_path, value).map(Some)
        }
    }
}

type ValueResult = Result<Value, RuntimeError>;

/// Make a koto Value out of some Rust value
pub trait IntoValue: Sized {
    fn into_value(self) -> ValueResult;
}

impl<T: IntoValue> IntoValue for Option<T> {
    fn into_value(self) -> ValueResult {
        self.map_or(Ok(Value::Empty), IntoValue::into_value)
    }
}

impl<T, E> IntoValue for Result<T, E>
where
    T: IntoValue,
    E: Error,
{
    fn into_value(self) -> ValueResult {
        match self {
            Ok(ok) => ok.into_value(),
            Err(e) => runtime_error!("{}", e),
        }
    }
}

impl IntoValue for () {
    fn into_value(self) -> ValueResult {
        Ok(Value::Empty)
    }
}

impl IntoValue for bool {
    fn into_value(self) -> ValueResult {
        Ok(Value::Bool(self))
    }
}

impl<T: IntoValue + Sized> IntoValue for Box<T> {
    fn into_value(self) -> ValueResult {
        (*self).into_value()
    }
}

impl<T: Clone + IntoValue> IntoValue for &T {
    fn into_value(self) -> ValueResult {
        self.clone().into_value()
    }
}

impl IntoValue for String {
    fn into_value(self) -> ValueResult {
        Ok(Value::Str(self.into()))
    }
}

impl IntoValue for &str {
    fn into_value(self) -> ValueResult {
        Ok(Value::Str(self.into()))
    }
}

macro_rules! impl_into_value_num {
    (one $ty:ty, $as_ty:ty, $variant:path) => {
        impl IntoValue for $ty {
            fn into_value(self) -> ValueResult {
                Ok(Value::Number($variant(self as $as_ty)))
            }
        }
    };
    ($variant:path, $as_ty:ty; $($tys:ty),*) => {
        $(
            impl_into_value_num!(one $tys, $as_ty, $variant);
        )*
    };
}

impl_into_value_num!(ValueNumber::I64, i64; u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);
impl_into_value_num!(ValueNumber::F64, f64; f32, f64);

mod chrono {
    // we cannot implement ExternalValue for chrono types, alas. So my
    // solution for now is to allow strings in standard format and

    use super::{field, FromValue, IntoValue, RefFromValue, Value, ValueResult};
    use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};

    macro_rules! impl_chrono_value {
        ($ty:ty, $tup_pat:pat, $ok:expr, $fmt:expr, $($accessors:ident),*) => {
            impl FromValue for $ty {
                fn from_value(
                    key_path: &crate::KeyPath<'_>,
                    value: &koto::runtime::Value
                ) -> Result<Self, koto::runtime::RuntimeError> {
                    match value {
                        Value::Tuple(tuple) => if let &$tup_pat = &tuple.data() {
                            Ok($ok)
                        } else {
                            crate::wrong_type(stringify!($ty), key_path, value)
                        },
                        Value::Str(s) => Self::parse_from_str(s.as_str(), $fmt)
                            .or_else(|e| ::koto::runtime::runtime_error!("{}", e)),
                        value => crate::wrong_type(stringify!($ty), key_path, value)
                    }
                }
            }

            impl RefFromValue for $ty {
                fn ref_from_value<R, F: Fn(&Self) -> R>(
                    key_path: &crate::KeyPath<'_>,
                    value: &Value,
                    f: F,
                ) -> Result<R, koto::runtime::RuntimeError> {
                    Ok(f(&Self::from_value(key_path, value)?))
                }
            }

            impl IntoValue for $ty {
                fn into_value(self) -> ValueResult {
                    #[allow(unused_imports)]
                    use ::chrono::{Datelike, Timelike};
                    Ok(Value::Tuple(::koto::runtime::ValueTuple::from(
                        &[$(self.$accessors().into_value()?),*][..]
                    )))
                }
            }
        };
    }

    impl_chrono_value!(
        NaiveDate,
        [y, m, d],
        NaiveDate::from_ymd(
            i32::from_value(&field("year"), &y)?,
            u32::from_value(&field("month"), &m)?,
            u32::from_value(&field("day"), &d)?
        ),
        "%Y-%m-%d",
        year,
        month,
        day
    );

    impl_chrono_value!(
        NaiveTime,
        [h, m, s],
        NaiveTime::from_hms(
            u32::from_value(&field("hour"), &h)?,
            u32::from_value(&field("minute"), &m)?,
            u32::from_value(&field("second"), &s)?
        ),
        "%H:%M:%S",
        hour,
        minute,
        second
    );

    impl_chrono_value!(
        NaiveDateTime,
        [y, m, d, h, min, s],
        NaiveDate::from_ymd(
            i32::from_value(&field("year"), &y)?,
            u32::from_value(&field("month"), &m)?,
            u32::from_value(&field("day"), &d)?
        )
        .and_hms(
            u32::from_value(&field("hour"), &h)?,
            u32::from_value(&field("minute"), &min)?,
            u32::from_value(&field("second"), &s)?
        ),
        "%Y-%m-%d %H:%M:%S",
        year,
        month,
        day,
        hour,
        minute,
        second
    );

    impl_chrono_value!(
        DateTime<FixedOffset>,
        [y, m, d, h, min, s, tz],
        DateTime::from_utc(
            NaiveDate::from_ymd(
                i32::from_value(&field("year"), &y)?,
                u32::from_value(&field("month"), &m)?,
                u32::from_value(&field("day"), &d)?
            )
            .and_hms(
                u32::from_value(&field("hour"), &h)?,
                u32::from_value(&field("minute"), &min)?,
                u32::from_value(&field("second"), &s)?
            ),
            FixedOffset::east(i32::from_value(&field("timezone"), &tz)?),
        ),
        "%Y %b %d %H:%M:%S%.3f %z",
        year,
        month,
        day,
        hour,
        minute,
        second,
        timezone
    );

    impl IntoValue for chrono::FixedOffset {
        fn into_value(self) -> ValueResult {
            Ok(Value::Empty) //TODO: Have to read up on what to do with an offset
        }
    }
}

use std::time::Duration;

/// Duration is either given as number of seconds (either float or integer) or a tuple of
/// (seconds, nanoseconds), both integer
impl FromValue for Duration {
    fn from_value(key_path: &KeyPath<'_>, value: &Value) -> Result<Self, RuntimeError> {
        match value {
            Value::Number(ValueNumber::I64(secs)) => return Ok(Duration::from_secs(*secs as u64)),
            Value::Number(ValueNumber::F64(secs)) => return Ok(Duration::from_secs_f64(*secs)),
            Value::Tuple(tup) => {
                if let [secs, nanos] = tup.data() {
                    return Ok(
                        Duration::from_secs(
                            u64::from_value(&field("seconds"), secs)?
                        ) + Duration::from_nanos(
                            u64::from_value(&field("nanoseconds"), nanos)?
                        ),
                    );
                }
            }
            _ => {}
        }
        wrong_type("Duration", key_path, value)
    }
}

impl RefFromValue for Duration {
    fn ref_from_value<R, F: Fn(&Self) -> R>(
        key_path: &KeyPath<'_>,
        value: &Value,
        f: F,
    ) -> Result<R, RuntimeError> {
        Ok(f(&Duration::from_value(key_path, value)?))
    }
}

/// Duration is always turned into a integer tuple of (seconds, nanoseconds)
impl IntoValue for Duration {
    fn into_value(self) -> ValueResult {
        Ok(Value::Tuple(koto::runtime::ValueTuple::from(
            &[
                self.as_secs().into_value()?,
                self.subsec_nanos().into_value()?,
            ][..],
        )))
    }
}

#[macro_export]
macro_rules! external_value {
    ($($ty:ty),*) => {
        $(
            impl $crate::StaticExternalValue for $ty {
                fn type_str() -> &'static str { stringify!($ty) }
            }

            impl ::koto::runtime::ExternalValue for $ty {
                fn value_type(&self) -> String {
                    String::from(stringify!($ty))
                }
            }

            impl $crate::FromValue for $ty {
                fn from_value(
                    key_path: &$crate::KeyPath<'_>,
                    value: &::koto::runtime::Value,
                ) -> std::result::Result<Self, ::koto::runtime::RuntimeError> {
                    if let Some(result) = <Self as $crate::CustomFromValue>::opt_from_value(value) {
                        return Ok(result);
                    }
                    if let ::koto::runtime::Value::ExternalValue(exval, ..) = value {
                        if let Some(v) = exval.as_ref().write().downcast_mut::<Self>() {
                            Ok(v.clone())
                        } else {
                            $crate::wrong_type(
                                <Self as $crate::StaticExternalValue>::type_str(),
                                key_path,
                                &value,
                            )
                        }
                    } else {
                        $crate::not_external_value(key_path, &*value)
                    }
                }
            }

            impl $crate::RefFromValue for $ty {
                fn ref_from_value<R, F: Fn(&Self) -> R>(
                    key_path: &$crate::KeyPath<'_>,
                    value: &::koto::runtime::Value,
                    f: F,
                ) -> Result<R, ::koto::runtime::RuntimeError> {
                    if let ::koto::runtime::Value::ExternalValue(exval, ..) = value {
                        if let Some(v) = exval.as_ref().read().downcast_ref::<Self>() {
                            Ok(f(v))
                        } else {
                            $crate::wrong_type(
                                <Self as $crate::StaticExternalValue>::type_str(),
                                key_path,
                                &value,
                            )
                        }
                    } else {
                        $crate::not_external_value(key_path, &*value)
                    }
                }

                fn ref_mut_from_value<R, F: for<'r> Fn(&'r Self) -> R>(
                    key_path: &$crate::KeyPath<'_>,
                    value: &::koto::runtime::Value,
                    f: F,
                ) -> Result<R, ::koto::runtime::RuntimeError> {
                    if let ::koto::runtime::Value::ExternalValue(exval, ..) = value {
                        if let Some(v) = exval.as_ref().write().downcast_mut::<Self>() {
                            Ok(f(v))
                        } else {
                            $crate::wrong_type(
                                <Self as $crate::StaticExternalValue>::type_str(),
                                key_path,
                                &value,
                            )
                        }
                    } else {
                        $crate::not_external_value(key_path, &*value)
                    }
                }
            }
        )*
    }
}