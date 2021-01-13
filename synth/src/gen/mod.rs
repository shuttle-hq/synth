use std::collections::HashMap;
use std::ops::Try;
use std::ops::Range;
use std::time::Duration;
use std::convert::TryFrom;
use std::fmt::Display;

use num::Zero;
use rand_regex::Regex as RandRegex;
use rand::distributions::{
    uniform::{SampleBorrow, SampleUniform, UniformDuration, UniformSampler},
    Bernoulli, Distribution, Uniform,
};
use anyhow::Result;
use compiler::NamespaceCompiler;
use rand::prelude::Rng as RandRng;

use synth_generator::Never;
use synth_generator::{
    error::Error as GeneratorError,
    prelude::*,
    value::{Map, Token, Tokenizer},
    Chain, Concatenate, GeneratorState, Just, Once, OneOf, Seed, Take,
};

pub mod compiler;
pub use compiler::{Compile, Compiler};

mod utils;
use utils::{Driver, Scoped, View};

use crate::schema::{Categorical, CategoricalType};
use crate::python::Pythonizer;
use crate::schema::{
    ChronoContent, ChronoContentFormatter, FakerContentArgument, Namespace, Range as SRange,
};

macro_rules! derive_generator {
    {
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
	    type Yield = Token;

	    type Return = ();

	    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
		let next = match self {
		    $(
			Self::$variant(inner) => inner.next(rng).into_yielded(),
		    )*
		};
		match next {
		    Err(_) => GeneratorState::Complete(()),
		    Ok(yielded) => GeneratorState::Yielded(yielded)
		}
	    }
	}
    }
}

impl Generator for Box<Model> {
    type Yield = <Model as Generator>::Yield;

    type Return = <Model as Generator>::Return;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        <Model as Generator>::next(self, rng)
    }
}

macro_rules! number_model {
    {
	$(
	    (
		$range:ident as $new_range:ident,
		$constant:ident as $new_constant:ident
	    ) for $ty:ty,
	)*
    } => {
	derive_generator!(
	    pub enum NumberModel {
		$(
		    $range(Tokenizer<Once<Seed<$ty, RangeDistribution<$ty>>>>),
		    $constant(Tokenizer<Just<$ty>>),
		)*
		U64Categorical(Tokenizer<Once<Seed<u64, Categorical<u64>>>>),
		I64Categorical(Tokenizer<Once<Seed<i64, Categorical<i64>>>>),
		U64Id(Tokenizer<Once<IncrementingSeed>>)
	    }
	);

	impl NumberModel {
	    $(
		pub fn $new_range(range: SRange<$ty>) -> Result<Self> {
		    let dist = RangeDistribution::try_from(range.clone())?;
		    let gen = Seed::new_with(dist)
			.once()
			.into_token();
		    Ok(Self::$range(gen))
		}

		pub fn $new_constant(value: $ty) -> Self {
		    let gen = value.yield_token();
		    Self::$constant(gen)
		}
	    )*
	}

    $(
    impl Distribution<$ty> for RangeDistribution<$ty> {
        fn sample<R: RandRng + ?Sized>(&self, rng: &mut R) -> $ty {
            let low = self.range.low;
            let high = self.range.high;
            let step = self.range.step;

            let temp = rng.gen_range(<$ty>::zero(), high - low);
            low + temp - (temp % step)
        }
    }
    )*
    }
}

number_model!(
    (U64Range as u64_range, U64Constant as u64_constant) for u64,
    (I64Range as i64_range, I64Constant as i64_constant) for i64,
    (F64Range as f64_range, F64Constant as f64_constant) for f64,
);

pub struct RangeDistribution<N: PartialOrd + Zero + Display> {
    range: SRange<N>,
}

impl<N: PartialOrd + Zero + Display> TryFrom<SRange<N>> for RangeDistribution<N> {
    type Error = anyhow::Error;

    fn try_from(range: SRange<N>) -> Result<Self> {
        if range.low >= range.high {
            return Err(
                failed!(target: Debug, BadRequest => "cannot create a distribution for a Range where 'low' ({}) is greater than or equal to 'high' ({}).", range.low, range.high),
            );
        }
        if range.step <= N::zero() {
            return Err(
                failed!(target: Debug, BadRequest => "cannot create a distribution for a Range where 'step' ({}) is less than or equal to 0.", range.step),
            );
        }
        Ok(Self { range })
    }
}

pub struct RandDateTime {
    inner: Uniform<ChronoContent>,
    format: String,
}

impl RandDateTime {
    pub fn new(range: Range<ChronoContent>, format: &str) -> Self {
        Self {
            inner: Uniform::new_inclusive(range.start, range.end),
            format: format.to_string(),
        }
    }
}

pub struct UniformChronoContent(ChronoContent, UniformDuration);

impl SampleUniform for ChronoContent {
    type Sampler = UniformChronoContent;
}

impl UniformSampler for UniformChronoContent {
    type X = ChronoContent;

    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        // safe because it has been asserted by rand API contract that	// high >= low, which implies same variant of ChronoContent
        let delta = low.borrow().delta_to(high.borrow()).unwrap();
        let inner = UniformDuration::new(Duration::default(), delta);
        UniformChronoContent(low.borrow().clone(), inner)
    }

    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        let delta = low.borrow().delta_to(high.borrow()).unwrap();
        let inner = UniformDuration::new_inclusive(Duration::default(), delta);
        UniformChronoContent(low.borrow().clone(), inner)
    }

    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        let delta = self.1.sample(rng);
        self.0.clone() + delta
    }
}

impl Distribution<Token> for RandDateTime {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Token {
        let inner = self.inner.sample(rng);
        ChronoContentFormatter::new(&self.format)
            .format(&inner)
            .map(|s| s.into_token())
            .unwrap_or_else(|err| GeneratorError::custom(err).into_token())
    }
}

derive_generator!(
    pub enum StringModel {
        Regex(Tokenizer<Once<Seed<String, RandRegex>>>),
        Chrono(Once<Seed<Token, RandDateTime>>),
        Categorical(Tokenizer<Once<Seed<String, Categorical<String>>>>),
    }
);

derive_generator!(
    pub enum BoolModel {
        Bernoulli(Tokenizer<Once<Seed<bool, Bernoulli>>>),
        Constant(Tokenizer<Just<bool>>),
        Categorical(Tokenizer<Once<Seed<bool, Categorical<bool>>>>),
    }
);

pub struct FakerSeed {
    pub generator: String,
    pub python: Pythonizer,
    pub args: HashMap<String, FakerContentArgument>,
}

pub struct IncrementingSeed {
    pub(crate) count: u64,
}

impl Generator for IncrementingSeed {
    type Yield = u64;
    type Return = Never;

    fn next(&mut self, _rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        self.count += 1;
        GeneratorState::Yielded(self.count - 1)
    }
}

impl Generator for FakerSeed {
    type Yield = Token;

    type Return = Never;

    fn next(&mut self, _rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        let mut inner = self.python.faker(&self.generator);
        for (key, value) in self.args.iter() {
            inner = inner.arg(key, value);
        }
        let generated = inner.generate::<String>().map(|g| g.into_token());
        GeneratorState::Yielded(
            generated.unwrap_or_else(|err| GeneratorError::custom(err).into_token()),
        )
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
    G::Return: Try,
    <G::Return as Try>::Error: IntoToken,
{
    type Yield = Token;
    type Return = ();

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
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
    pub enum PrimitiveModel {
        Bool(BoolModel),
        Number(NumberModel),
        String(StringModel),
        Faker(Once<FakerSeed>),
        Null(Tokenizer<Just<()>>),
        Error(Tokenizer<Just<GeneratorError>>),
    }
);

derive_generator!(
    pub enum Model {
        Primitive(PrimitiveModel),
        Object(Map<Chain<Concatenate<Tokenizer<Just<String>>, Model>>>),
        Array(Box<dyn Generator<Return = (), Yield = Token>>),
        OneOf(OneOf<Model>),
        Optional(OneOf<Box<Model>>),
        Driver(Driver<Model>),
        View(Unwrapped<View<Model>>),
        Scoped(Scoped<Model>),
        Many(Take<Box<Model>>),
    }
);

/// @brokad: use primitives instead, this is hacky...
pub struct Unwrapped<G> {
    inner: G,
    is_complete: bool,
}

impl Unwrapped<View<Model>> {
    fn wrap(inner: View<Model>) -> Self {
        Self {
            inner,
            is_complete: false,
        }
    }
}

impl Generator for Unwrapped<View<Model>> {
    type Yield = Token;

    type Return = ();

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        if self.is_complete {
            self.is_complete = false;
            GeneratorState::Complete(())
        } else {
            match self.inner.next(rng) {
                GeneratorState::Yielded(yielded) => GeneratorState::Yielded(yielded),
                GeneratorState::Complete(complete) => {
                    self.is_complete = true;
                    match complete {
                        Some(()) => self.next(rng),
                        None => GeneratorState::Yielded(Token::Primitive(Primitive::Null(()))),
                    }
                }
            }
        }
    }
}

impl<T: CategoricalType> Distribution<T> for Categorical<T> {
    fn sample<R: RandRng + ?Sized>(&self, rng: &mut R) -> T {
        let f = rng.gen_range(0.0, 1.0);
        let mut index = (f * self.total as f64).floor() as i64;
        for (k, v) in self.seen.iter() {
            index -= *v as i64;
            if index < 0 {
                return k.clone();
            }
        }
        unreachable!()
    }
}

impl Model {
    pub fn null() -> Self {
        Model::Primitive(PrimitiveModel::Null(().yield_token()))
    }

    pub fn from_namespace(ns: &Namespace, pythonizer: &Pythonizer) -> Result<Self> {
        NamespaceCompiler::new(ns, pythonizer).compile()
    }
}

#[cfg(test)]
pub mod tests {
    use std::collections::HashSet;

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
            "start" : 100
            }
            },
            "is_active": {
            "type": "bool",
            "frequency": 0.2
            },
            "username": {
            "type": "string",
            "pattern": "[a-z0-9]{5,15}"
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
            "type": "one_of",
            "variants": [ {
                "type": "string",
                "faker": {
                "generator": "currency_name",
                }
            }, {
                "type": "string",
                "pattern": "unknown"
            }, ]
            },
            "credit_card": {
            "type": "string",
            "faker": {
                "generator": "credit_card_number",
                "card_type": "amex"
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
                "generator": "ascii_email"
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
            }
        }
            }
        });

        let mut rng = rand::thread_rng();

        let python = Pythonizer::new().unwrap();

        let mut model = Model::from_namespace(&schema, &python)
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
        }

        #[allow(dead_code)]
        #[derive(Deserialize, Debug)]
        struct SampleUserData {
            id: u64,
            num_logins: u64,
            username: String,
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
            let ser = OwnedSerializable::new(model.complete(&mut rng));
            let generated_str = serde_json::to_string_pretty(&ser).unwrap();

            let sample_data = serde_json::from_str::<SampleData>(&generated_str).unwrap();

            let mut all_users = HashSet::new();
            let mut currencies = HashMap::new();
            for user in sample_data.users {
                assert_eq!(user.num_logins, user.num_logins_again);
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
            }

            for value in counts.values() {
                assert_eq!(*value, 10);
            }
        }
    }

    #[test]
    fn test_schema_compiles_and_generates() {
        let python = Pythonizer::new().unwrap();
        let mut model = Model::from_namespace(&USER_NAMESPACE, &python)
            .unwrap()
            .aggregate();
        let mut rng = rand::thread_rng();
        let ser = OwnedSerializable::new(model.complete(&mut rng));
        serde_json::to_string_pretty(&ser).unwrap();
    }

    #[test]
    fn range_distribution_u64() {
        let range = SRange::<u64>::new(15, 40, 5);
        let dist = RangeDistribution::try_from(range).unwrap();
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
        let range = SRange::<i64>::new(-10, 10, 5);
        let dist = RangeDistribution::try_from(range).unwrap();
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
        let range = SRange::new(-2.5, 1.0, 1.5);
        let dist = RangeDistribution::try_from(range).unwrap();
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
        let range = SRange::<u64>::new(10, 10, 5);
        assert!(RangeDistribution::try_from(range).is_err())
    }

    #[test]
    fn range_distribution_step_larger_than_delta() {
        let range = SRange::<u64>::new(10, 15, 10);
        let dist = RangeDistribution::try_from(range).unwrap();
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
        let range = SRange::<u64>::new(10, 15, 5);
        let dist = RangeDistribution::try_from(range).unwrap();
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
