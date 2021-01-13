//! **DEPRECATED** Will be replaced by [`read`](crate::read).
#![allow(unused_variables, unused_imports, missing_docs)]
use std::convert::{TryFrom, TryInto};
use std::iter::Peekable;

use std::marker::PhantomData;

use serde::{
    de::{
        EnumAccess as SerdeEnumAccess, MapAccess as SerdeMapAccess, SeqAccess as SerdeSeqAccess,
        VariantAccess as SerdeVariantAccess,
    },
    Deserialize, Deserializer as SerdeDeserializer,
};

use crate::{value::*, Error, Generator, GeneratorState, PeekableGenerator, Rng};

trait TokenIterator<'de>
where
    Self: Iterator<Item = &'de Token>,
{
    fn next_or_bail(&mut self) -> Result<&'de Token, Error> {
        self.next()
            .ok_or(<Error as serde::de::Error>::custom("unexpected: EOF"))
    }

    fn clone_next_or_bail(&mut self) -> Result<Token, Error> {
        self.next_or_bail().map(|token| token.clone())
    }
}

impl<'de, I> TokenIterator<'de> for I where I: Iterator<Item = &'de Token> {}

macro_rules! deserialize_number {
    ($($type_:ident,)*) => {$(deserialize_number_helper!($type_);)*};
}

macro_rules! deserialize_number_helper {
    (i8) => (deserialize_number_impl!(i8 => deserialize_i8 visit_i8););
    (i16) => (deserialize_number_impl!(i16 => deserialize_i16 visit_i16););
    (i32) => (deserialize_number_impl!(i32 => deserialize_i32 visit_i32););
    (i64) => (deserialize_number_impl!(i64 => deserialize_i64 visit_i64););
    (i128) => (deserialize_number_impl!(i128 => deserialize_i128 visit_i128););
    (u8) => (deserialize_number_impl!(u8 => deserialize_u8 visit_u8););
    (u16) => (deserialize_number_impl!(u16 => deserialize_u16 visit_u16););
    (u32) => (deserialize_number_impl!(u32 => deserialize_u32 visit_u32););
    (u64) => (deserialize_number_impl!(u64 => deserialize_u64 visit_u64););
    (u128) => (deserialize_number_impl!(u128 => deserialize_u128 visit_u128););
    (f32) => (deserialize_number_impl!(f32 => deserialize_f32 visit_f32););
    (f64) => (deserialize_number_impl!(f64 => deserialize_f64 visit_f64););
}

macro_rules! deserialize_number_impl {
    ($type_:ty => $de:ident $visit:ident) => {
        fn $de<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            self.0
                .clone_next_or_bail()?
                .try_into()
                .and_then(|p: Primitive| p.try_into())
                .and_then(|n: Number| visitor.$visit(n.try_into()?))
        }
    };
}

macro_rules! deserialize_value {
    ($($type_:ty => $de:ident $visit:ident,)*) => {
	$(deserialize_value!($type_ => $de $visit);)*
    };
    ($type_:ty => $de:ident $visit:ident) => {
        fn $de<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
	    self.0
		.clone_next_or_bail()?
		.try_into()
		.and_then(|p: Primitive| visitor.$visit(p.try_into()?))
        }
    };
}

macro_rules! deserialize_redirect {
    ($($de:ident -> $rde:ident,)*) => {
	$(deserialize_redirect!($de -> $rde);)*
    };
    ($de:ident -> $rde:ident) => {
	fn $de<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
            V: serde::de::Visitor<'de>,
	{
	    self.$rde(visitor)
	}
    };
}

pub struct MapAccess<'i, 'de, I: Iterator<Item = &'de Token>>(&'i mut Peekable<I>);

impl<'i, 'de, I> SerdeMapAccess<'de> for MapAccess<'i, 'de, I>
where
    I: Iterator<Item = &'de Token>,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        match self.0.peek() {
            Some(Token::Special(Special::EndMap)) | Some(Token::Special(Special::EndStruct)) => {
                Ok(None)
            }
            _ => seed
                .deserialize(Deserializer(self.0))
                .map(|value| Some(value)),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(Deserializer(self.0))
    }
}

pub struct SeqAccess<'i, 'de, I: Iterator<Item = &'de Token>>(&'i mut Peekable<I>);

impl<'i, 'de, I> SerdeSeqAccess<'de> for SeqAccess<'i, 'de, I>
where
    I: Iterator<Item = &'de Token>,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.0.peek() {
            Some(Token::Special(Special::EndSeq)) => Ok(None),
            _ => seed
                .deserialize(Deserializer(self.0))
                .map(|value| Some(value)),
        }
    }
}

pub struct EnumAccess<'i, 'de, I: Iterator<Item = &'de Token>>(&'i mut Peekable<I>);

impl<'i, 'de, I> SerdeEnumAccess<'de> for EnumAccess<'i, 'de, I>
where
    I: Iterator<Item = &'de Token>,
{
    type Error = Error;

    type Variant = VariantAccess<'i, 'de, I>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let value = seed.deserialize(Deserializer(self.0))?;
        Ok((value, VariantAccess(self.0)))
    }
}

pub struct VariantAccess<'i, 'de, I: Iterator<Item = &'de Token>>(&'i mut Peekable<I>);

impl<'i, 'de, I> SerdeVariantAccess<'de> for VariantAccess<'i, 'de, I>
where
    I: Iterator<Item = &'de Token>,
{
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        match self.0.next() {
            Some(Token::Primitive(Primitive::Null(..))) => Ok(()),
            _ => todo!(),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(Deserializer(self.0))
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Deserializer(self.0).deserialize_tuple(len, visitor)
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Deserializer(self.0).deserialize_struct("", fields, visitor)
    }
}

pub struct Deserializer<'i, 'de, I: Iterator<Item = &'de Token>>(&'i mut Peekable<I>);

impl<'i, 'de, I> SerdeDeserializer<'de> for Deserializer<'i, 'de, I>
where
    I: Iterator<Item = &'de Token>,
{
    type Error = Error;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if let Some(next) = self.0.peek() {
            match next {
                Token::Primitive(primitive) => match primitive {
                    Primitive::Bool(_) => self.deserialize_bool(visitor),
                    Primitive::String(_) => self.deserialize_string(visitor),
                    Primitive::Char(_) => self.deserialize_char(visitor),
                    Primitive::Null(_) => self.deserialize_unit(visitor),
                    Primitive::Bytes(_) => self.deserialize_bytes(visitor),
                    Primitive::Number(_) => todo!(),
                },
                Token::Special(special) => match special {
                    Special::BeginMap(_) => self.deserialize_map(visitor),
                    Special::BeginStruct(s, n) => self.deserialize_struct(s, &[], visitor),
                    Special::BeginTuple(n) => self.deserialize_tuple(*n, visitor),
                    Special::BeginSeq(_) => self.deserialize_seq(visitor),
                    Special::BeginSome | Special::None => self.deserialize_option(visitor),
                    Special::UnitStruct(s) => self.deserialize_unit_struct(*s, visitor),
                    otherwise => Err(Self::Error::custom(format!("unexpected: {}", otherwise))),
                },
            }
        } else {
            Err(Self::Error::custom("unexpected: EOF"))
        }
    }

    deserialize_value!(
    bool => deserialize_bool visit_bool,
    char => deserialize_char visit_char,
    Vec<u8> => deserialize_byte_buf visit_byte_buf,
    );

    deserialize_number!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64,);

    deserialize_redirect!(
    deserialize_str -> deserialize_string,
    deserialize_bytes -> deserialize_byte_buf,
    );

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let next = self.0.clone_next_or_bail()?;
        match next {
            Token::Primitive(Primitive::String(s)) => visitor.visit_string(s),
            Token::Special(Special::BeginField(s)) => visitor.visit_str(s),
            otherwise => Err(Error::type_("string", otherwise)),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.0.peek() {
            Some(Token::Primitive(Primitive::Null(..))) => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.0.next() {
            Some(Token::Primitive(Primitive::Null(..))) => Ok(()),
            _ => todo!(),
        }?;
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        // @brokad: not sure this is right but this is how serde_json does it
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.0.clone_next_or_bail()?.begin_seq()?;
        let seq_access = SeqAccess(self.0);
        let value = visitor.visit_seq(seq_access)?;
        self.0.clone_next_or_bail()?.end_seq()?;
        Ok(value)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        // @brokad: not sure this is right but this is how serde_json does it
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        // @brokad: not sure this is right but this is how serde_json does it
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.0.clone_next_or_bail()?.begin_map()?;
        let map_access = MapAccess(self.0);
        let value = visitor.visit_map(map_access)?;
        self.0.clone_next_or_bail()?.end_map()?;
        Ok(value)
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.0.clone_next_or_bail()?.begin_struct()?;
        let map_access = MapAccess(self.0);
        let value = visitor.visit_map(map_access)?;
        self.0.clone_next_or_bail()?.end_struct()?;
        Ok(value)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.0.clone_next_or_bail()?.begin_struct()?;
        let enum_access = EnumAccess(self.0);
        let value = visitor.visit_enum(enum_access)?;
        self.0.clone_next_or_bail()?.end_struct()?;
        Ok(value)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }
}

pub struct Deserializator<G: Generator<Yield = Token>, T>
where
    for<'de> T: Deserialize<'de>,
{
    pub(crate) inner: G,
    pub(crate) buf: Vec<Token>,
    pub(crate) _output: PhantomData<T>,
}

impl<G, T> Generator for Deserializator<G, T>
where
    G: Generator<Yield = Token>,
    for<'de> T: Deserialize<'de>,
{
    type Yield = Token;

    type Return = Result<T, Error>;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        match self.inner.next(rng) {
            GeneratorState::Yielded(token) => {
                self.buf.push(token.clone());
                GeneratorState::Yielded(token)
            }
            GeneratorState::Complete(_) => {
                let buf = std::mem::replace(&mut self.buf, Vec::new());
                let mut iter = buf.iter().peekable();
                let de = Deserializer(&mut iter);
                GeneratorState::Complete(T::deserialize(de))
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
}
