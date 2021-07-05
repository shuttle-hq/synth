use std::collections::BTreeMap;

use anyhow::{Context, Result};

use synth_gen::prelude::*;
use synth_gen::value::{Token, Tokenizer};

use crate::compile::NamespaceCompiler;
use crate::compile::{Driver, Scoped, View};

use crate::schema::{ChronoValue, Namespace};

macro_rules! derive_generator {
    {
	yield $yield:ty,
	return $return:ty,
	$vis:vis enum $id:ident {
	    $(
		$variant:ident($inner:ty$(,)?)$(,)?
	    )*
	}
    } => {
	$vis enum $id {
	    $($variant($inner),)*
	}

	impl Generator for $id {
	    type Yield = $yield;

	    type Return = $return;

	    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
		match self {
		    $(
			Self::$variant(inner) => inner.next(rng),
		    )*
		}
	    }
	}
    };
    {
	yield $yield:ty,
	return $return:ty,
	$vis:vis struct $id:ident($inner:ty);
    } => {
	$vis struct $id($inner);

	impl Generator for $id {
	    type Yield = $yield;

	    type Return = $return;

	    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
		self.0.next(rng)
	    }
	}
    }
}

pub mod prelude;
use prelude::*;

pub mod null;
pub use null::NullNode;

pub mod string;
pub use string::{RandFaker, RandomDateTime, RandomString, StringNode, Truncated, UuidGen};

pub mod number;
pub use number::{Incrementing, NumberNode, RandomF64, RandomI64, RandomU64, UniformRangeStep};

pub mod boolean;
pub use boolean::{BoolNode, RandomBool};

pub mod array;
pub use array::ArrayNode;

pub mod object;
pub use object::{KeyValueOrNothing, ObjectNode};


pub mod unique;
pub use unique::UniqueNode;

pub mod one_of;
pub(crate) mod series;

use crate::graph::series::SeriesNode;
pub use one_of::OneOfNode;

pub type JustToken<T> = Tokenizer<Just<T>>;

pub type TokenOnce<T> = Tokenizer<Once<T>>;

pub type Valuize<G, T> =
    MapComplete<G, fn(Result<T, Error>) -> Result<Value, Error>, Result<Value, Error>>;

pub type Devaluize<G, T> =
    MapComplete<G, fn(Result<Value, Error>) -> Result<T, Error>, Result<T, Error>>;

pub type OwnedDevaluize<G, T> = Exhaust<Devaluize<G, T>>;

pub type OnceInfallible<G> = TryOnce<Infallible<G, Error>>;

macro_rules! derive_from {
    {
	#[$attr:meta]
	$vis:vis enum $id:ident {
	    $($variant:ident$(($ty:ty))?,)*
	}
    } => {
	#[$attr]
	$vis enum $id {
	    $($variant$(($ty))?,)*
	}

	impl $id {
	    pub fn type_(&self) -> &'static str {
		match self {
		    $(Self::$variant(_) => stringify!($variant),)*
		}
	    }
	}

	$(
	    $(
		impl From<$ty> for $id {
		    fn from(value: $ty) -> Self {
			Self::$variant(value)
		    }
		}

		impl TryInto<$ty> for $id {
		    type Error = Error;
		    fn try_into(self) -> Result<$ty, Self::Error> {
			match self {
			    Self::$variant(value) => Ok(value),
			    otherwise => Err(
				failed_crate!(
				    target: Release,
				    "invalid type: expected '{}', found '{}'",
				    stringify!($variant),
				    otherwise.type_()
				)
			    )
			}
		    }
		}
	    )?
	)*
    };
}

pub fn value_from_ok<T>(value: Result<T, Error>) -> Result<Value, Error>
where
    Value: From<T>,
{
    value.map(Value::from)
}

pub fn value_from_ok_number<T>(value: Result<T, Error>) -> Result<Value, Error>
where
    Number: From<T>,
{
    value.map(|t| Number::from(t).into())
}

pub fn number_from_ok<T>(value: Result<Value, Error>) -> Result<T, Error>
where
    T: TryFrom<Number>,
    T::Error: std::error::Error,
{
    value.and_then(|v| v.try_into()).and_then(|n: Number| {
        n.try_into().map_err(|err| {
            failed_crate!(target: Release, "could not convert from '{}': {}", n, err)
        })
    })
}

derive_from! {
    #[derive(Debug, Clone, Hash, PartialEq)]
    pub enum Value {
        Null(()),
        Bool(bool),
        Number(Number),
        String(String),
        DateTime(ChronoValue),
        Object(BTreeMap<String, Value>),
        Array(Vec<Value>),
    }
}

pub struct IntoCompleted<G> {
    inner: G,
    complete: bool,
}

impl<G> IntoCompleted<G> {
    pub fn wrap(inner: G) -> Self {
        Self {
            inner,
            complete: false,
        }
    }
}

impl<G> Generator for IntoCompleted<G>
where
    G: Generator<Yield = Token>,
    G::Return: GeneratorResult,
    <G::Return as GeneratorResult>::Err: IntoToken,
{
    type Yield = Token;
    type Return = ();

    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        if std::mem::replace(&mut self.complete, false) {
            GeneratorState::Complete(())
        } else {
            match self.inner.next(rng) {
                GeneratorState::Yielded(y) => GeneratorState::Yielded(y),
                GeneratorState::Complete(r) => match r.into_result() {
                    Ok(_) => {
                        self.complete = true;
                        self.next(rng)
                    }
                    Err(r_err) => {
                        self.complete = true;
                        GeneratorState::Yielded(r_err.into_token())
                    }
                },
            }
        }
    }
}

derive_generator!(
    yield Token,
    return Result<Value, Error>,
    pub enum Graph {
        Null(NullNode),
        Bool(BoolNode),
        Number(NumberNode),
        String(StringNode),
        Object(ObjectNode),
        Array(ArrayNode),
        OneOf(OneOfNode),
        Driver(Driver<Graph>),
        View(Unwrapped<View<Graph>>),
        Scoped(Scoped<Graph>),
        Series(SeriesNode),
        Unique(UniqueNode)
    }
);

pub type BoxedGraph = Box<Graph>;

impl Generator for Box<Graph> {
    type Yield = <Graph as Generator>::Yield;

    type Return = <Graph as Generator>::Return;

    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        <Graph as Generator>::next(self, rng)
    }
}

/// @brokad: use primitives instead, this is hacky...
pub struct Unwrapped<G> {
    inner: G,
    is_complete: bool,
    value: Option<Result<Value, Error>>,
}

impl Unwrapped<View<Graph>> {
    pub fn wrap(inner: View<Graph>) -> Self {
        Self {
            inner,
            is_complete: false,
            value: None,
        }
    }
}

impl Generator for Unwrapped<View<Graph>> {
    type Yield = Token;

    type Return = Result<Value, Error>;

    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        if self.is_complete {
            self.is_complete = false;
            let value = std::mem::replace(&mut self.value, None).unwrap();
            GeneratorState::Complete(value)
        } else {
            match self.inner.next(rng) {
                GeneratorState::Yielded(yielded) => GeneratorState::Yielded(yielded),
                GeneratorState::Complete(complete) => {
                    self.is_complete = true;
                    match complete {
                        Some(value) => {
                            self.value = Some(value);
                            self.next(rng)
                        }
                        None => GeneratorState::Yielded(Token::Primitive(Primitive::Null(()))),
                    }
                }
            }
        }
    }
}

impl Graph {
    pub fn null() -> Self {
        Graph::Null(
            ().yield_token()
                .infallible()
                .map_complete(value_from_ok::<()>),
        )
    }

    pub fn from_namespace(ns: &Namespace) -> Result<Self> {
        NamespaceCompiler::new(ns)
            .compile()
            .context("while compiling the namespace")
    }
}

#[cfg(test)]
pub mod tests {
    use std::collections::HashSet;

    use rand::thread_rng;

    use super::*;

    use crate::schema::tests::USER_NAMESPACE;

    #[test]
    fn schema_to_generator() {
        let schema: Namespace = from_json!({
            "users": {
        "type": "array",
        "length": {
            "type": "number",
            "subtype": "u64",
            "constant": 10
        },
        "content": {
            "type": "object",
            "id" : {
            "type" : "number",
            "subtype" : "u64",
            "id" : {
            "start_at" : 100
            }
            },
            "is_active": {
            "type": "bool",
            "frequency": 0.2
            },
            "username": {
                "type": "string",
                "truncated": {
                    "content": {
                        "type": "string",
                        "pattern": "[a-zA-Z0-9]{0, 255}"
                    },
                    "length": 5
                }
            },
            "bank_country": {
            "type": "string",
            "pattern": "(GB|ES)"
                    },
            "num_logins": {
            "type": "number",
            "subtype": "u64",
            "range": {
                "high": 100,
                "low": 0,
                "step": 1
            }
            },
            "currency": {
            "type": "string",
            "pattern": "(USD|GBP)"
                    },
            "credit_card": {
            "type": "string",
                "faker": {
                    "generator": "credit_card"
                }
            },
            "created_at_date": {
            "type": "string",
            "date_time": {
                "format": "%Y-%m-%d"
            }
            },
            "created_at_time": {
            "type": "string",
            "date_time": {
                "format": "%H:%M:%S"
            }
            },
            "last_login_at": {
            "type": "string",
            "date_time": {
                "format": "%Y-%m-%dT%H:%M:%S%z",
                "begin": "2020-01-01T00:00:00+0000"
            }
            },
            "maybe_an_email": {
            "optional": true,
            "type": "string",
            "faker": {
                "generator": "safe_email"
            }
            },
            "num_logins_again": {
            "type": "same_as",
            "ref": "users.content.num_logins"
            }
                }
            },
            "transactions": {
        "type": "array",
        "length": {
            "type": "number",
            "subtype": "u64",
            "constant": 100
        },
        "content": {
            "type": "object",
            "username": {
            "type": "same_as",
            "ref": "users.content.username"
            },
            "currency": {
            "type": "same_as",
            "ref": "users.content.currency"
            },
            "timestamp": {
            "type": "string",
            "date_time": {
                "format": "%Y-%m-%dT%H:%M:%S%z",
                "begin": "2020-01-01T00:00:00+0000"
            }
            },
            "amount": {
            "type": "number",
            "subtype": "f64",
            "range": {
                "high": 10000,
                "low": 0,
                "step": 0.1
            }
            },
            "serialized_nonce": {
            "type" : "string",
            "serialized" : {
                "serializer" : "json",
                 "content" : {
                 "type" : "object",
                 "nonce" : {
                     "type" : "string",
                     "pattern" : "[A-Z a-z 0-9]+",
                }
                }
             }
            },
        }
            }
        });

        let mut rng = rand::thread_rng();

        let mut model = Graph::from_namespace(&schema)
            .unwrap()
            .inspect(|yielded| {
                println!("{:?}", yielded);
            })
            .aggregate();

        #[derive(Deserialize, Debug)]
        struct SampleData {
            users: Vec<SampleUserData>,
            transactions: Vec<SampleTransactionData>,
        }

        #[allow(dead_code)]
        #[derive(Deserialize, Debug)]
        struct SampleTransactionData {
            username: String,
            currency: String,
            timestamp: String,
            amount: f64,
            serialized_nonce: String,
        }

        #[allow(dead_code)]
        #[derive(Deserialize, Debug)]
        struct SampleUserData {
            id: u64,
            num_logins: u64,
            username: String,
            bank_country: String,
            currency: String,
            credit_card: String,
            maybe_an_email: Option<String>,
            is_active: bool,
            created_at_date: String,
            created_at_time: String,
            last_login_at: String,
            num_logins_again: u64,
        }

        for _ in 0..100 {
            let ser = OwnedSerializable::new(model.try_next_yielded(&mut rng).unwrap());
            let generated_str = serde_json::to_string_pretty(&ser).unwrap();

            let sample_data = serde_json::from_str::<SampleData>(&generated_str).unwrap();

            let mut all_users = HashSet::new();
            let mut currencies = HashMap::new();
            for user in sample_data.users {
                assert_eq!(user.num_logins, user.num_logins_again);
                println!("bank_country={}", user.bank_country);
                assert!(&user.bank_country == "GB" || &user.bank_country == "ES");
                assert!(user.id >= 100);
                assert!(user.username.len() <= 5);
                all_users.insert(user.username.clone());
                currencies.insert(user.username, user.currency);
                /*
                       if let Some(email) = user.maybe_an_email {
                           if !user.is_active {
                               assert!(
                                   email.contains("inactive"),
                                   "email did not contain inactive: {}",
                                   email
                               )
                           }
                       }

                       ChronoContentFormatter::new("%Y-%m-%d")
                           .parse(&user.created_at_date)
                           .unwrap();

                       ChronoContentFormatter::new("%H:%M:%S")
                           .parse(&user.created_at_time)
                           .unwrap();

                       ChronoContentFormatter::new("%Y-%m-%dT%H:%M:%S%z")
                           .parse(&user.last_login_at)
                           .unwrap();
                */
            }
            assert_eq!(all_users.len(), 10);

            println!("currencies={:?}", currencies);

            let mut counts = HashMap::new();
            for transaction in sample_data.transactions {
                println!("transaction={:?}", transaction);
                assert!(all_users.contains(&transaction.username));
                println!(
                    "username={}, amount={}",
                    transaction.username, transaction.amount
                );
                assert_eq!(
                    transaction.currency,
                    *currencies.get(&transaction.username).unwrap()
                );
                *counts.entry(transaction.username).or_insert(0) += 1;

                assert!(serde_json::to_value(transaction.serialized_nonce).is_ok());
            }

            for value in counts.values() {
                assert_eq!(*value, 10);
            }
        }
    }

    #[test]
    fn test_schema_compiles_and_generates() {
        let mut model = Graph::from_namespace(&USER_NAMESPACE).unwrap().aggregate();
        let mut rng = rand::thread_rng();
        let ser = OwnedSerializable::new(model.try_next_yielded(&mut rng).unwrap());
        serde_json::to_string_pretty(&ser).unwrap();
    }

    #[test]
    fn range_distribution_u64() {
        let range = RangeStep::<u64>::new(15, 40, 5);
        let dist = UniformRangeStep::try_from(range).unwrap();
        let mut rng = thread_rng();
        for _ in 1..100 {
            match dist.sample(&mut rng) {
                15 => {}
                20 => {}
                25 => {}
                30 => {}
                35 => {}
                n => {
                    panic!("Generated '{}' which should not happen", n)
                }
            }
        }
    }

    #[test]
    fn range_distribution_i64() {
        let range = RangeStep::<i64>::new(-10, 10, 5);
        let dist = UniformRangeStep::try_from(range).unwrap();
        let mut rng = thread_rng();
        for _ in 1..100 {
            match dist.sample(&mut rng) {
                -10 => {}
                -5 => {}
                0 => {}
                5 => {}
                n => {
                    panic!("Generated '{}' which should not happen", n)
                }
            }
        }
    }

    #[test]
    fn range_distribution_f64() {
        let range = RangeStep::new(-2.5, 1.0, 1.5);
        let dist = UniformRangeStep::try_from(range).unwrap();
        let mut rng = thread_rng();
        for _ in 1..1000 {
            let sample = dist.sample(&mut rng);
            // Not using pattern matching here because of  <https://github.com/rust-lang/rust/issues/41620>.
            // As of 2020-12-01 it causes a linter warning which will be a compiler error in future releases.
            if sample == -2.5 || sample == -1.0 || sample == 0.5 {
            } else {
                panic!("Generated '{}' which should not happen", sample)
            }
        }
    }

    #[test]
    fn range_distribution_constant() {
        let range = RangeStep::<u64>::new(10, 10, 5);
        assert!(UniformRangeStep::try_from(range).is_err())
    }

    #[test]
    fn range_distribution_step_larger_than_delta() {
        let range = RangeStep::<u64>::new(10, 15, 10);
        let dist = UniformRangeStep::try_from(range).unwrap();
        let mut rng = thread_rng();
        for _ in 1..100 {
            match dist.sample(&mut rng) {
                10 => {}
                n => {
                    panic!("Generated '{}' which should not happen", n)
                }
            }
        }
    }

    #[test]
    fn range_distribution_step_is_delta() {
        let range = RangeStep::<u64>::new(10, 15, 5);
        let dist = UniformRangeStep::try_from(range).unwrap();
        let mut rng = thread_rng();
        for _ in 1..100 {
            match dist.sample(&mut rng) {
                10 => {}
                n => {
                    panic!("Generated '{}' which should not happen", n)
                }
            }
        }
    }
}
