use super::prelude::*;

use rand::distributions::uniform::SampleUniform;

use num::{One, Zero, CheckedAdd};

use std::ops::{Add, Sub, Rem};

pub struct UniformRangeStep<N>(RangeStep<N>);

impl<N: PartialOrd + Zero + Display> TryFrom<RangeStep<N>> for UniformRangeStep<N> {
    type Error = anyhow::Error;

    fn try_from(range: RangeStep<N>) -> Result<Self, Self::Error> {
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
        Ok(Self(range))
    }
}

impl<N> Distribution<N> for UniformRangeStep<N>
where
    N: Zero + Add<Output = N> + Sub<Output = N> + Rem<Output = N> + SampleUniform + Copy,
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> N {
	let low = self.0.low;
	let high = self.0.high;
	let step = self.0.step;

	let temp: N = rng.gen_range(N::zero(), high - low);
	low + temp - (temp % step)
    }
}

pub struct Incrementing<N = i64> {
    count: N,
    step: N,
}

impl<N> Incrementing<N>
where
    N: Zero + One
{
    pub fn new() -> Self {
	Self {
	    count: N::zero(),
	    step: N::one()
	}
    }
}

impl<N> Incrementing<N>
where
    N: One
{
    pub fn new_at(start_at: N) -> Self {
	Self {
	    count: start_at,
	    step: N::one()
	}
    }
}

impl<N> Incrementing<N> {
    pub fn new_at_by(start_at: N, step: N) -> Self {
	Self {
	    count: start_at,
	    step
	}
    }
}

impl<N> Generator for Incrementing<N>
where
    N: CheckedAdd + Copy
{
    type Yield = N;

    type Return = Result<Never, Error>;

    fn next<R: Rng>(&mut self, _rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
	match self.count.checked_add(&self.step) {
	    Some(next) => {
		self.count = next;
		GeneratorState::Yielded(next)
	    },
	    None => GeneratorState::Complete(Err(failed_crate!(
		target: Release,
		"incrementing generator overflowed: try using a larger type"
	    )))
	}
    }
}

macro_rules! number_node {
    {
	$(
	    $rand:ident (
		$range:ident as $new_range:ident,
		$constant:ident as $new_constant:ident,
		$(
		    $categorical:ident as $new_categorical:ident
		)?,
		$(
		    $incrementing:ident as $new_incrementing:ident
		)?,
	    ) for $ty:ty,
	)*
    } => {
	$(
	    derive_generator! {
		yield $ty,
		return Result<$ty, Error>,
		pub enum $rand {
		    $range(OnceInfallible<Random<$ty, UniformRangeStep<$ty>>>),
		    $constant(OnceInfallible<Yield<$ty>>),
		    $($categorical(OnceInfallible<Random<$ty, Categorical<$ty>>>),)?
		    $($incrementing(TryOnce<Incrementing<$ty>>),)?
		}
	    }
	)*
	    
	$(
	    impl $rand {
		pub fn $new_range(range: RangeStep<$ty>) -> Result<Self, anyhow::Error> {
		    let dist = UniformRangeStep::try_from(range)?;
		    Ok(Self::$range(Random::new_with(dist).infallible().try_once()))
		}

		pub fn $new_constant(value: $ty) -> Self {
		    Self::$constant(Yield::wrap(value).infallible().try_once())
		}

		$(
		    pub fn $new_categorical(cat: Categorical<$ty>) -> Self {
			Self::$categorical(Random::new_with(cat).infallible().try_once())
		    }
		)?

		$(
		    pub fn $new_incrementing(incr: Incrementing<$ty>) -> Self {
			Self::$incrementing(incr.try_once())
		    }
		)?
	    }
	)*

	derive_generator! {
	    yield Token,
	    return Result<Value, Error>,
	    pub enum NumberNode {
		$(
		    $rand(Valuize<Tokenizer<$rand>, $ty>),
		)*
	    }
	}

	$(
	    impl From<$rand> for NumberNode {
		fn from(value: $rand) -> Self {
		    Self::$rand(value.into_token().map_complete(value_from_ok_number::<$ty>))
		}
	    }
	)*
    }
}

number_node!(
    RandomU64 (
    U64Range as range,
    U64Constant as constant,
    U64Categorical as categorical,
    Incrementing as incrementing,
    ) for u64,
    RandomI64 (
    I64Range as range,
    I64Constant as constant,
    I64Categorical as categorical,
    Incrementing as incrementing,
    ) for i64,
    RandomF64 (
    F64Range as range,
    F64Constant as constant,,,
    ) for f64,
);
