//! Custom tokenization for the serde data model.
//!
use std::convert::{TryFrom, TryInto};
use std::marker::PhantomData;

use rand::Rng;

use crate::{
    de::Deserializator,
    generator::{Brace, Concatenate, Just, Prefix, Yield},
    Error, Generator, GeneratorExt, GeneratorState,
};

use ordered_float::OrderedFloat;
use serde::Deserialize;

pub type OrderedFloat32 = OrderedFloat<f32>;
pub type OrderedFloat64 = OrderedFloat<f64>;

macro_rules! generate_enum {
    {
        $(#[$attr:meta])*
        $vis:vis enum $id:ident {
            $(
                $(#[$variant_attr:meta])*
                $variant:ident($type_:ty),
            )*
        }
    } => {
        $(#[$attr])*
        $vis enum $id {
            $(
                $(#[$variant_attr])*
                $variant($type_),
            )*
        }

        impl std::fmt::Display for $id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$variant(inner) => write!(f, "{:?}", inner),
                    )*
                }
            }
        }

        impl std::fmt::Debug for $id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$variant(inner) => write!(f, "{}({:?})", stringify!($type_), inner),
                    )*
                }
            }
        }

        $(
            impl From<$type_> for $id {
                fn from(value: $type_) -> Self {
                    Self::$variant(value)
                }
            }
        )*

        $(
            impl TryFrom<$id> for $type_ {
                type Error = Error;
                fn try_from(value: $id) -> Result<$type_, <Self as TryFrom<$id>>::Error> {
                    match value {
                        $id::$variant(value) => Ok(value),
                        otherwise => Err(Error::r#type(stringify!($variant), otherwise))
                    }
                }
            }
        )*
	
        impl $id {
            pub fn type_(&self) -> &'static str {
                match self {
                    $( Self::$variant(_) => stringify!($variant), )*
                }
            }
        }	
    };
}

macro_rules! is_variant {
    {
        #[$name:expr]
        $item:item
    } => {
        #[doc = "Check if the token is an instance of `"]
        #[doc = $name]
        #[doc = "`."]
        $item
    }
}

macro_rules! to_variant {
    {
        #[$name:expr]
        $item:item
    } => {
        #[doc = "Consume the token and extract out the arguments for `"]
        #[doc = $name]
        #[doc = "`."]
        /// # Errors
        /// If `self` is not an instance of a `
        #[doc = $name]
        #[doc = "` "]
        /// token, a custom error of type `S::Error` is returned.
        $item
    }
}

macro_rules! generate_special_enum {
    {
        $(#[$attr:meta])*
        $vis:vis enum $id:ident {
            $(
                $(#[$variant_attr:meta])*
                $variant:ident$(($($arg:ty as $var:ident,)*))? -> $is:ident $to:ident,
            )*
        }
    } => {
        $(#[$attr])*
        $vis enum $id {
            $(
                $(#[$variant_attr])*
                $variant$(($($arg,)*))*,
            )*
        }

        impl std::fmt::Display for $id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$variant$(($(drop!($arg),)*))? =>
                            write!(f, "{}", stringify!($variant)),
                    )*
                }
            }
        }

        impl Token {
            $(
                is_variant! {
                    #[stringify!($variant)]
                    pub fn $is(&self) -> bool {
                        match self {
                            Self::$id($id::$variant$(($(drop!($arg),)*))?) => true,
                            _ => false
                        }
                    }
                }

                to_variant! {
                    #[stringify!($variant)]
                    pub fn $to<E: serde::ser::Error>(self) -> Result<($($($arg,)*)?), E> {
                        match self {
                            Token::$id($id::$variant$(($($var,)*))?) => Ok(($($($var,)*)*)),
                            otherwise => {
                                let err = format!(
                                    "unexpected: wanted {}, got {:?}",
                                    stringify!($variant),
                                    otherwise
                                );
                                Err(E::custom(err))
                            }
                        }
                    }
                }
            )*
        }

        /// **DEPRECATED** Used by the
        /// [`bynar::de::Deserializer`](crate::de::Deserializer).
        #[allow(missing_docs)]
        pub trait SpecialTokenExt: Generator<Yield = Token> {
            $(
                #[inline]
                fn $to<R: Rng>(&mut self, rng: &mut R) -> Result<(), Error> {
                    match self.next(rng).into_yielded()?.try_into()? {
                        $id::$variant$(($(drop!($arg),)*))? => Ok(()),
                        otherwise => Err(Error::r#type(stringify!($variant), otherwise))
                    }
                }
            )*
        }

        impl<G> SpecialTokenExt for G where G: Generator<Yield = Token> {}
    };
}

macro_rules! drop {
    { $arg:ty } => { _ }
}

macro_rules! into_composite {
    {
        $name:expr,
        $(#[$attr:meta])*,
        $item:item
    } => {
        /// Make `self` a composite generator of type `
        #[doc = $name]
        #[doc = "`. See"]
        $(#[$attr])*
        #[doc = " for documentation on the arguments."]
        $item
    }
}

macro_rules! data_model_variant_impl_ext {
    {
        $(
            $(#[$attr:meta])*
            $as:ident($($arg:ident: $ty:ty$(,)?)*) -> $id:ident$(,)?
        )*
    } => {
        $(
            into_composite! {
                stringify!($id),
                $(#[$attr])*,
                fn $as(self, $($arg: $ty,)*) -> $id<Self> {
                    $id::new(self, $($arg,)*)
                }
            }
        )*
    }
}

generate_enum!(
    /// Token variant for serde primitive numerical types.
    #[allow(missing_docs)]
    #[derive(Clone, Copy, Hash, PartialEq, Eq)]
    pub enum Number {
        I8(i8),
        I16(i16),
        I32(i32),
        I64(i64),
        I128(i128),
        U8(u8),
        U16(u16),
        U32(u32),
        U64(u64),
        U128(u128),
        F32(OrderedFloat32),
        F64(OrderedFloat64),
    }
);

impl From<f32> for Number {
    fn from(f: f32) -> Self {
        Number::F32(OrderedFloat(f))
    }
}

impl From<f64> for Number {
    fn from(f: f64) -> Self {
        Number::F64(OrderedFloat(f))
    }
}

impl TryFrom<Number> for f32 {
    type Error = Error;
    fn try_from(value: Number) -> Result<f32, <Self as TryFrom<Number>>::Error> {
        match value {
            Number::F32(of32) => Ok(of32.into_inner()),
            otherwise => Err(Error::r#type(stringify!(F32), otherwise)),
        }
    }
}

impl TryFrom<Number> for f64 {
    type Error = Error;
    fn try_from(value: Number) -> Result<f64, <Self as TryFrom<Number>>::Error> {
        match value {
            Number::F64(of64) => Ok(of64.into_inner()),
            otherwise => Err(Error::r#type(stringify!(F64), otherwise)),
        }
    }
}

impl serde::Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            Self::I8(x) => serializer.serialize_i8(x),
            Self::I16(x) => serializer.serialize_i16(x),
            Self::I32(x) => serializer.serialize_i32(x),
            Self::I64(x) => serializer.serialize_i64(x),
            Self::I128(x) => serializer.serialize_i128(x),
            Self::U8(x) => serializer.serialize_u8(x),
            Self::U16(x) => serializer.serialize_u16(x),
            Self::U32(x) => serializer.serialize_u32(x),
            Self::U64(x) => serializer.serialize_u64(x),
            Self::U128(x) => serializer.serialize_u128(x),
            Self::F32(x) => serializer.serialize_f32(x.into_inner()),
            Self::F64(x) => serializer.serialize_f64(x.into_inner()),
        }
    }
}

generate_enum!(
    /// Token variant for all serde primitive (non-composite) types.
    #[allow(missing_docs)]
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub enum Primitive {
        Bool(bool),
        String(String),
        Char(char),
        Bytes(Vec<u8>),
        Number(Number),
        Null(()),
    }
);

impl serde::Serialize for Primitive {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Bool(b) => serializer.serialize_bool(*b),
            Self::String(s) => serializer.serialize_str(&s),
            Self::Char(c) => serializer.serialize_char(*c),
            Self::Bytes(b) => serializer.serialize_bytes(b.as_slice()),
            Self::Null(()) => serializer.serialize_unit(),
            Self::Number(number) => number.serialize(serializer),
        }
    }
}

generate_special_enum!(
    /// Token variant for serde control flow statements that allow
    /// for building instances of composite types.
    #[allow(missing_docs)]
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub enum Special {
        BeginMap(Option<usize> as len,) -> is_begin_map begin_map,
        EndMap -> is_end_map end_map,
        BeginStruct(&'static str as s, usize as n,) -> is_begin_struct begin_struct,
        EndStruct -> is_end_struct end_struct,
        BeginField(&'static str as s,) -> is_begin_field begin_field,
        BeginTuple(usize as n,) -> is_begin_tuple begin_tuple,
        EndTuple -> is_end_tuple end_tuple,
        BeginSeq(Option<usize> as n,) -> is_begin_seq begin_seq,
        EndSeq -> is_end_seq end_seq,
        BeginSome -> is_begin_some begin_some,
        None -> is_none none,
        UnitStruct(&'static str as s,) -> is_unit_struct unit_struct,
        UnitVariant(&'static str as n, u32 as i, &'static str as s,) -> is_unit_variant unit_variant,
        Error(Error as error,) -> is_error error,
    }
);

generate_enum!(
    /// A custom tokenization for the serde data model.
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub enum Token {
        /// A token encoding serde primitive types.
        Primitive(Primitive),
        /// A token encoding a serde control flow statement for composite types.
        Special(Special),
    }
);

impl Token {
    /// Transform this [`Token`](Token) into a constant
    /// [`Generator`](crate::Generator).
    pub fn just(self) -> Just<Self> {
        Yield::wrap(self).once()
    }
}

/// A generator that yields and returns a single token.
pub type OneToken = Just<Token>;

/// Alias trait for [`Generator`](crate::Generator)s that yield values
/// of type [`Token`](Token).
pub trait TokenGenerator: Generator<Yield = Token> {}

impl<G> TokenGenerator for G where G: Generator<Yield = Token> {}

/// Helper extension to manufacture generators for composite serde
/// data types from [`TokenGenerator`](crate::value::TokenGenerator).
pub trait TokenGeneratorExt: TokenGenerator + Sized {
    data_model_variant_impl_ext! {
        /// [`serialize_field`](serde::ser::SerializeStruct::serialize_field)
        into_struct_field(name: &'static str) -> StructField,
        /// [`serialize_struct`](serde::ser::Serializer::serialize_struct)
        into_struct(name: &'static str, len: usize) -> Struct,
        /// [`serialize_seq`](serde::ser::Serializer::serialize_seq)
        into_seq(len: Option<usize>) -> Seq,
        /// [`serialize_tuple`](serde::ser::Serializer::serialize_tuple)
        into_tuple(len: usize) -> Tuple,
        /// [`serialize_map`](serde::ser::Serializer::serialize_map)
        into_map(len: Option<usize>) -> Map,
    }

    fn with_key<K: TokenGenerator>(self, key: K) -> Concatenate<K, Self> {
        key.concatenate(self)
    }

    fn deserialize<T>(self) -> Deserializator<Self, T>
    where
        for<'de> T: Deserialize<'de>,
    {
        Deserializator {
            inner: self,
            buf: Vec::new(),
            _output: PhantomData,
        }
    }
}

impl<G> TokenGeneratorExt for G where G: TokenGenerator {}

/// Types that have a specified conversion to [`Token`](Token).
pub trait IntoToken: Sized {
    /// Convert `self` to [`Token`](Token).
    fn into_token(self) -> Token;

    /// Same as `Yield::new(self).once().into_token()`.
    #[inline]
    fn yield_token(self) -> Tokenizer<Just<Self>>
    where
        Self: Clone,
    {
        Yield::wrap(self).once().into_token()
    }
}

impl IntoToken for Token {
    fn into_token(self) -> Token {
        self
    }
}

macro_rules! impl_into_token {
    {
        $(
            $track:ident: $($ty:ty$(,)?)+ => $fst:ty $(=> $inter:ty)* $(,)?
        )*
    } => {
        $(
            #[inline]
            fn $track<I>(x: I) -> Token
            where
                $fst: From<I>
            {
                let out: $fst = x.into();
                $(let out: $inter = out.into();)*
                out
            }

            $(
                impl IntoToken for $ty {
                    #[inline]
                    fn into_token(self) -> Token {
                        $track(self)
                    }
                }
            )+
        )*
    }
}

impl_into_token! {
    token_from_number:
    i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64, OrderedFloat32, OrderedFloat64 => Number => Primitive => Token,
    token_from_primitives: bool => Primitive => Token,
    token_from_string: String, &str => String => Primitive => Token,
    token_from_null: () => Primitive => Token,
}

/// Helper trait for [`Generator`](crate::Generator)s that yield
/// values of a type implementing [`IntoToken`](IntoToken).
pub trait IntoTokenGeneratorExt: Generator + Sized
where
    <Self as Generator>::Yield: IntoToken,
{
    /// Convert this [`Generator`](crate::Generator) into a
    /// [`TokenGenerator`](TokenGenerator)
    fn into_token(self) -> Tokenizer<Self> {
        Tokenizer { inner: self }
    }
}

/// A token generator that applies
/// [`into_token`](IntoToken::into_token) to values yielded by an
/// other one.
pub struct Tokenizer<G>
where
    G: Generator,
    G::Yield: IntoToken,
{
    inner: G,
}

impl<G> Tokenizer<G>
where
    G: Generator,
    G::Yield: IntoToken,
{
    pub fn into_inner(self) -> G {
        self.inner
    }
}

impl<G> Generator for Tokenizer<G>
where
    G: Generator,
    G::Yield: IntoToken,
{
    type Yield = Token;

    type Return = G::Return;

    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        match self.inner.next(rng) {
            GeneratorState::Yielded(y) => GeneratorState::Yielded(y.into_token()),
            GeneratorState::Complete(r) => GeneratorState::Complete(r),
        }
    }
}

impl<G> IntoTokenGeneratorExt for G
where
    G: Generator + Sized,
    G::Yield: IntoToken,
{
}

macro_rules! data_model_variant {
    {
        $name:expr,
        $serde:expr,
        $id:ident<G>($($arg:ident: $ty:ty$(,)?)*) -> $inner:ty = $cl:expr
    } => {
        #[doc = $name]
        pub struct $id<G>
        where
            G: Generator<Yield = Token>
        {
            pub inner: $inner
        }

        impl<G> $id<G>
        where
            G: Generator<Yield = Token>,
        {
            /// Create a new instance from a generator `G`. See
            #[doc = $serde]
            /// for documentation on the additional arguments.
            pub fn new(generator: G, $($arg: $ty,)*) -> Self {
                Self {
                    inner: ($cl)(generator)
                }
            }
        }

        impl<G> Generator for $id<G>
        where
            G: Generator<Yield = Token>,
        {
            type Yield = G::Yield;

            type Return = G::Return;

            fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
                self.inner.next(rng)
            }
        }
    }
}

data_model_variant! {
    "A generator of [struct fields](https://serde.rs/data-model.html).",
    "[`serialize_field`](serde::ser::SerializeStruct::serialize_field)",
    StructField<G>(name: &'static str) -> Prefix<OneToken, G> = |g: G| g.prefix(
        Token::Special(Special::BeginField(name)).just()
    )
}

data_model_variant! {
    "A generator of [structs](https://serde.rs/data-model.html).",
    "[`serialize_struct`](serde::ser::Serializer::serialize_struct)",
    Struct<G>(name: &'static str, len: usize) -> Brace<OneToken, G, OneToken> = |g: G| g.brace(
        Token::Special(Special::BeginStruct(name, len)).just(),
        Token::Special(Special::EndStruct).just(),
    )
}

data_model_variant! {
    "A generator of [tuples](https://serde.rs/data-model.html).",
    "[`serialize_tuple`](serde::ser::Serializer::serialize_tuple)",
    Tuple<G>(len: usize) -> Brace<OneToken, G, OneToken> = |g: G| g.brace(
        Token::Special(Special::BeginTuple(len)).just(),
        Token::Special(Special::EndTuple).just(),
    )
}

data_model_variant! {
    "A generator of [seqs](https://serde.rs/data-model.html).",
    "[`serialize_seq`](serde::ser::Serializer::serialize_seq)",
    Seq<G>(len: Option<usize>) -> Brace<OneToken, G, OneToken> = |g: G| g.brace(
        Token::Special(Special::BeginSeq(len)).just(),
        Token::Special(Special::EndSeq).just(),
    )
}

data_model_variant! {
    "A generator of [maps](https://serde.rs/data-model.html).",
    "[`serialize_map`](serde::ser::Serializer::serialize_map)",
    Map<G>(len: Option<usize>) -> Brace<OneToken, G, OneToken> = |g: G| g.brace(
        Token::Special(Special::BeginMap(len)).just(),
        Token::Special(Special::EndMap).just(),
    )
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{ser::OwnedSerializable, GeneratorExt};

    use crate::generator::Random;

    pub fn base_gen(i: usize) -> impl Generator<Yield = Token, Return = ()> {
        (i as u32)
            .yield_token()
            .into_struct_field("an_u32")
            .and_then(|value| {
                value
                    .to_string()
                    .yield_token()
                    .into_struct_field("a_string")
                    .map_complete(move |_| value)
            })
            .into_struct("NestedStruct", 2)
            .into_struct_field("a_struct")
            .and_then(|an_u32| {
                true.yield_token()
                    .repeat(an_u32 as usize)
                    .into_seq(Some(an_u32 as usize))
                    .into_struct_field("a_seq")
            })
            .into_struct("BaseGen", 2)
            .map_complete(|_| ())
    }

    macro_rules! test_primitive_values {
        {
            $(
                $id:ident<$ty:ty>$(,)?
            )*
        } => {
            $(
                #[test]
                fn $id() {
                    let mut rng = rand::thread_rng();
                    let mut seed = Random::new::<$ty>()
                        .once()
                        .into_token()
                        .aggregate();
                    let next = seed.next(&mut rng).into_yielded().unwrap();
                    let as_ser = OwnedSerializable::new(next);
                    let as_str = serde_json::to_string(&as_ser).unwrap();
                    let _as_num: $ty = serde_json::from_str(&as_str).unwrap();
                }
            )*
        }
    }

    test_primitive_values!(
        value_i8<i8>,
        value_i16<i16>,
        value_i32<i32>,
        value_i64<i64>,
        value_i128<i128>,
        value_u8<u8>,
        value_u16<u16>,
        value_u32<u32>,
        value_u64<u64>,
        value_u128<u128>,
        value_f32<OrderedFloat32>,
        value_f64<OrderedFloat64>,
        value_bool<bool>,
    );

    #[test]
    fn deserializator() {
        #[derive(Debug, Deserialize, PartialEq, Eq)]
        struct BaseGen {
            a_struct: NestedStruct,
            a_seq: Vec<bool>,
        }

        #[derive(Debug, Deserialize, PartialEq, Eq)]
        struct NestedStruct {
            an_u32: u32,
            a_string: String,
        }

        let mut rng = rand::thread_rng();
        let base_gen = base_gen(7)
            .inspect(|gened| println!("{:?}", gened))
            .deserialize::<BaseGen>()
            .complete(&mut rng)
            .unwrap();

        assert_eq!(base_gen.a_seq.len(), 7);
        assert_eq!(base_gen.a_struct.an_u32, 7);
        assert_eq!(base_gen.a_struct.a_string, "7");
    }
}
