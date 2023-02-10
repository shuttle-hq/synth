//! This module provides a wrapper called
//! [`OwnedSerializable`](crate::ser::OwnedSerializable) around iterables of
//! [`Token`](crate::value::Token).
//!
//! This is used to convert completed streams of tokens into a
//! [`Serialize`](serde::Serialize) object that can be ingested by
//! serde's [`Serializer`](serde::ser::Serializer) implementations.
use serde::ser::Error;

use crate::value::{Special, Token};

use serde::ser::{SerializeMap, SerializeSeq, SerializeStruct};
use std::cell::RefCell;
use std::iter::Peekable;

#[derive(Debug)]
struct Hidden<I: Iterator<Item = impl std::fmt::Debug>>(RefCell<Peekable<I>>);

impl<I> Hidden<I>
where
    I: Iterator,
    I::Item: std::fmt::Debug,
{
    fn next<E>(&self) -> Result<I::Item, E>
    where
        E: serde::ser::Error,
    {
        self.0
            .borrow_mut()
            .next()
            .ok_or(E::custom("unexpected: EOF"))
    }

    fn next_if<F>(&self, cond: F) -> Option<I::Item>
    where
        F: FnOnce(&I::Item) -> bool,
    {
        let mut inner = self.0.borrow_mut();
        match inner.peek() {
            Some(item) if cond(item) => inner.next(),
            _ => None,
        }
    }

    fn peek<F, O>(&self, cond: F) -> Option<O>
    where
        F: FnOnce(&I::Item) -> O,
    {
        self.0.borrow_mut().peek().map(|value| cond(value))
    }
}

/// A wrapper around an iterator of [`Token`](crate::value::Token)s
/// that implement [`Serialize`](serde::Serialize).
#[derive(Debug)]
pub struct OwnedSerializable<I: Iterator> {
    inner: Hidden<I>,
}

impl<I: Iterator> OwnedSerializable<I> {
    /// Constructs a new [`OwnedSerializable`](OwnedSerializable) from
    /// an [`IntoIterator`](std::iter::IntoIterator).
    pub fn new<II>(into_iter: II) -> Self
    where
        II: IntoIterator<Item = I::Item, IntoIter = I>,
    {
        Self {
            inner: Hidden(RefCell::new(into_iter.into_iter().peekable())),
        }
    }
}

impl<V> std::iter::FromIterator<V> for OwnedSerializable<std::vec::IntoIter<V>> {
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        let buf: Vec<_> = iter.into_iter().collect();
        Self::new(buf)
    }
}

impl<I> serde::Serialize for OwnedSerializable<I>
where
    I: Iterator<Item = Token>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let ser = Serializable { inner: &self.inner };
        ser.serialize(serializer)
    }
}

struct Serializable<'a, I: Iterator> {
    inner: &'a Hidden<I>,
}

impl<'a, I> serde::Serialize for Serializable<'a, I>
where
    I: Iterator<Item = Token>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.inner.next::<S::Error>()? {
            Token::Special(special) => match special {
                Special::BeginStruct(name, len) => {
                    let s_s = SerializableStruct {
                        name,
                        len,
                        inner: self.inner,
                    };
                    s_s.serialize(serializer)
                }
                Special::BeginMap(len) => {
                    let s_s = SerializableMap {
                        len,
                        inner: self.inner,
                    };
                    s_s.serialize(serializer)
                }
                Special::None => serializer.serialize_none(),
                Special::BeginSome => {
                    let ser = Serializable { inner: self.inner };
                    serializer.serialize_some(&ser)
                }
                Special::UnitStruct(name) => serializer.serialize_unit_struct(name),
                Special::UnitVariant(name, variant_index, variant) => {
                    serializer.serialize_unit_variant(name, variant_index, variant)
                }
                Special::BeginSeq(len) => {
                    let ser = SerializableSeq {
                        len,
                        inner: self.inner,
                    };
                    ser.serialize(serializer)
                }
                Special::Error(error) => Err(S::Error::custom(error)),
                otherwise => {
                    let err = format!(
                        "Stream of generated tokens is incomplete: the generated stream is malformed. Expected new data, instead got: {}",
                        otherwise
                    );
                    Err(S::Error::custom(err))
                }
            },
            Token::Primitive(primitive) => primitive.serialize(serializer),
        }
    }
}

struct SerializableMap<'a, I: Iterator> {
    len: Option<usize>,
    inner: &'a Hidden<I>,
}

impl<'a, I> serde::Serialize for SerializableMap<'a, I>
where
    I: Iterator<Item = Token>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s_s = serializer.serialize_map(self.len)?;
        while let Some(true) = self.inner.peek(|t| !t.is_end_map()) {
            let key = Serializable { inner: self.inner };
            s_s.serialize_key(&key)?;
            let value = Serializable { inner: self.inner };
            s_s.serialize_value(&value)?;
        }
        self.inner
            .next::<S::Error>()
            .and_then(|t| t.end_map::<S::Error>())?;
        s_s.end()
    }
}

struct SerializableSeq<'a, I: Iterator> {
    len: Option<usize>,
    inner: &'a Hidden<I>,
}

impl<'a, I> serde::Serialize for SerializableSeq<'a, I>
where
    I: Iterator<Item = Token>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s_s = serializer.serialize_seq(self.len)?;
        while let Some(true) = self.inner.peek(|t| !t.is_end_seq()) {
            let rest = Serializable { inner: self.inner };
            s_s.serialize_element(&rest)?;
        }
        self.inner
            .next::<S::Error>()
            .and_then(|t| t.end_seq::<S::Error>())?;
        s_s.end()
    }
}

struct SerializableStruct<'a, I: Iterator> {
    name: &'static str,
    len: usize,
    inner: &'a Hidden<I>,
}

impl<'a, I> serde::Serialize for SerializableStruct<'a, I>
where
    I: Iterator<Item = Token>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s_s = serializer.serialize_struct(self.name, self.len)?;
        while let Some(token) = self.inner.next_if(|t| !t.is_end_struct()) {
            let (name,) = token.begin_field::<S::Error>()?;
            let rest = Serializable { inner: self.inner };
            s_s.serialize_field(name, &rest)?;
        }
        self.inner
            .next::<S::Error>()
            .and_then(|t| t.end_struct::<S::Error>())?;
        s_s.end()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::value::tests::base_gen;
    use crate::{Generator, GeneratorExt};

    #[test]
    fn serialize() {
        let mut rng = rand::thread_rng();
        for i in 0..10 {
            let mut gen = base_gen(i).aggregate();
            let next = gen.next(&mut rng).into_yielded().unwrap();
            let ser = OwnedSerializable::new(next);
            let as_str = serde_json::to_string_pretty(&ser).unwrap();
            let as_value: serde_json::Value = serde_json::from_str(&as_str).unwrap();
            assert_eq!(
                as_value,
                serde_json::json!({
                    "a_struct": {
                    "an_u32": i,
                    "a_string": i.to_string()
                    },
                    "a_seq": std::iter::repeat(true).take(i).collect::<Vec<_>>()
                })
            )
        }
    }
}
