use crate::graph::prelude::content::series::SeriesVariant;
use crate::graph::prelude::*;
use crate::schema::content::series::SeriesContent;
use anyhow::Result;
use chrono::{Duration, NaiveDateTime};
use std::f64::consts::PI;
use std::ops::Add;

derive_generator! {
    yield Token,
    return Result<Value, Error>,
    pub enum SeriesNode {
        Incrementing(Valuize<Tokenizer<OnceInfallible<FormattedIncrementing>>, String>),
        Poisson(Valuize<Tokenizer<OnceInfallible<FormattedPoisson>>, String>),
        Cyclical(Valuize<Tokenizer<OnceInfallible<FormattedCyclical>>, String>),
        Zip(Valuize<Tokenizer<OnceInfallible<FormattedZip>>, String>)
    }
}

type FormattedIncrementing = SeriesFormatter<IncrementingSeries<NaiveDateTime, Duration>>;
type FormattedPoisson = SeriesFormatter<PoissonSeries>;
type FormattedCyclical = SeriesFormatter<CyclicalSeries>;
type FormattedZip = SeriesFormatter<ZipSeries>;

impl TryFrom<&SeriesContent> for SeriesNode {
    type Error = anyhow::Error;

    fn try_from(series_content: &SeriesContent) -> Result<Self> {
        let default_pattern = "%Y-%m-%d %H:%M:%S";
        let fmt = series_content.format.as_deref().unwrap_or(default_pattern);
        let sn = match &series_content.variant {
            SeriesVariant::Incrementing(incrementing) => {
                let incrementing_series = SeriesVariant::inc(incrementing, fmt)?;
                let sf = SeriesFormatter {
                    inner: incrementing_series,
                    format: fmt.to_string(),
                };
                Self::Incrementing(
                    sf.infallible()
                        .try_once()
                        .into_token()
                        .map_complete(value_from_ok::<String>),
                )
            }
            SeriesVariant::Poisson(poisson) => {
                let poisson_series = SeriesVariant::poisson(poisson, fmt)?;
                let sf = SeriesFormatter {
                    inner: poisson_series,
                    format: fmt.to_string(),
                };
                Self::Poisson(
                    sf.infallible()
                        .try_once()
                        .into_token()
                        .map_complete(value_from_ok::<String>),
                )
            }
            SeriesVariant::Cyclical(cyclical) => {
                let cyclical_series = SeriesVariant::cyclical(cyclical, fmt)?;
                let sf = SeriesFormatter {
                    inner: cyclical_series,
                    format: fmt.to_string(),
                };
                Self::Cyclical(
                    sf.infallible()
                        .try_once()
                        .into_token()
                        .map_complete(value_from_ok::<String>),
                )
            }
            SeriesVariant::Zip(zip) => {
                let zip_series = SeriesVariant::zip(zip, fmt)?;
                let sf = SeriesFormatter {
                    inner: zip_series,
                    format: fmt.to_string(),
                };
                Self::Zip(
                    sf.infallible()
                        .try_once()
                        .into_token()
                        .map_complete(value_from_ok::<String>),
                )
            }
        };
        Ok(sn)
    }
}

pub enum TimeSeries {
    Incrementing(IncrementingSeries<NaiveDateTime, Duration>),
    Poisson(PoissonSeries),
    Cyclical(CyclicalSeries),
    Zip(ZipSeries),
}

impl Generator for TimeSeries {
    type Yield = NaiveDateTime;
    type Return = Never;

    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        match self {
            TimeSeries::Poisson(poisson) => poisson.next(rng),
            TimeSeries::Cyclical(cyclical) => cyclical.next(rng),
            TimeSeries::Zip(zip) => zip.next(rng),
            TimeSeries::Incrementing(incrementing) => incrementing.next(rng),
        }
    }
}

pub struct SeriesFormatter<S> {
    inner: S,
    format: String,
}

impl<S> Generator for SeriesFormatter<S>
where
    S: Generator<Yield = NaiveDateTime, Return = Never>,
{
    type Yield = String;
    type Return = S::Return;

    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        match self.inner.next(rng) {
            GeneratorState::Yielded(v) => {
                GeneratorState::Yielded(v.format(&self.format).to_string())
            }
            GeneratorState::Complete(c) => GeneratorState::Complete(c),
        }
    }
}

pub struct IncrementingSeries<T, I> {
    current: T,
    increment: I,
}

impl<T, I> IncrementingSeries<T, I> {
    pub fn new(current: T, increment: I) -> Self {
        Self { current, increment }
    }
}

impl<T, I> Generator for IncrementingSeries<T, I>
where
    T: Add<I, Output = T> + Clone,
    I: Clone,
{
    type Yield = T;
    type Return = Never;

    fn next<R: Rng>(&mut self, _rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        let next = self.current.clone() + self.increment.clone();
        let current = std::mem::replace(&mut self.current, next);
        GeneratorState::Yielded(current)
    }
}

// https://preshing.com/20111007/how-to-generate-random-timings-for-a-poisson-process/
pub struct PoissonSeries {
    current: NaiveDateTime, // Todo change to Synth chrono value
    rate: Duration,
}

impl PoissonSeries {
    pub fn new(start: NaiveDateTime, rate: Duration) -> Self {
        Self {
            current: start,
            rate,
        }
    }
}

impl Generator for PoissonSeries {
    type Yield = NaiveDateTime;
    type Return = Never;

    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        let delta = self.rate.num_milliseconds() as f64
            * (-1.0 * (1.0 - rng.gen_range(0.0f64..1.0f64)).ln());
        self.current += chrono::Duration::milliseconds(delta as i64);
        GeneratorState::Yielded(self.current)
    }
}

// Todo explain what this does and how it works
pub struct CyclicalSeries {
    start: NaiveDateTime,
    current: NaiveDateTime,
    period: Duration,
    min_rate: Duration,
    max_rate: Duration,
}

impl CyclicalSeries {
    pub fn new(
        start: NaiveDateTime,
        period: Duration,
        min_rate: Duration,
        max_rate: Duration,
    ) -> Self {
        Self {
            start,
            current: start,
            period,
            min_rate,
            max_rate,
        }
    }
}

impl Generator for CyclicalSeries {
    type Yield = NaiveDateTime;
    type Return = Never;

    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        let start_ms = self.start.timestamp_millis();
        let current_ms = self.current.timestamp_millis();
        let period_ms = self.period.num_milliseconds();
        let max_rate_ms = self.max_rate.num_milliseconds();
        let min_rate_ms = self.min_rate.num_milliseconds();
        let phase = 2.0 * PI * ((current_ms - start_ms) % period_ms) as f64 / period_ms as f64;
        let delta = 1.0 + (min_rate_ms as f64 + ((max_rate_ms - min_rate_ms) as f64 * phase.sin()));
        let next_delta = delta * (-1.0 * (1.0 - rng.gen_range(0.0f64..1.0f64)).ln());
        self.current += chrono::Duration::milliseconds(next_delta as i64);
        GeneratorState::Yielded(self.current)
    }
}

// A composite series which zips together values from its children
pub struct ZipSeries {
    children: Vec<Peek<TimeSeries>>,
}

impl ZipSeries {
    pub fn new(children: Vec<TimeSeries>) -> anyhow::Result<Self> {
        if children.is_empty() {
            return Err(anyhow!("Cannot instantiate a Zip Series with 0 children"));
        }
        Ok(ZipSeries {
            children: children.into_iter().map(|g| g.peekable()).collect(),
        })
    }
}

impl Generator for ZipSeries {
    type Yield = NaiveDateTime;
    type Return = Never;

    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        let (mut earliest_index, mut earliest) = (0, &chrono::naive::NaiveDateTime::MAX);
        for (child_index, child) in self.children.iter_mut().enumerate() {
            let next = child.peek_next(rng).as_ref().into_yielded().unwrap();
            if next < earliest {
                earliest_index = child_index;
                earliest = next;
            }
        }
        self.children.get_mut(earliest_index).unwrap().next(rng)
    }
}

/// v_t = C + \sum_{i=1}^{N} \alpha_i v_{t - i} + \sum_{j=0}^{M} \beta_j \epsilon_{t-j} , \beta_0 = 1
pub struct AutoCorrelatedSeries {
    alpha: Vec<Duration>,
    beta: Vec<Duration>,
    v: Vec<NaiveDateTime>,
    eps: Vec<f64>,
    constant: NaiveDateTime,
}

impl Generator for AutoCorrelatedSeries {
    type Yield = NaiveDateTime;
    type Return = Never;

    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        let mut current = self.constant.timestamp_millis();
        for i in 0..self.alpha.len() {
            let t = self.v.len();
            let delta = self.alpha.get(i).unwrap().num_milliseconds()
                * self
                    .v
                    .get(t - i)
                    .map(|val| val.timestamp_millis())
                    .unwrap_or(0);
            current += delta;
        }

        self.eps.push(rng.gen());

        for j in 0..self.beta.len() {
            let t = self.eps.len();
            let delta = self.beta.get(j).unwrap().num_milliseconds() as f64
                * self.eps.get(t - j).unwrap_or(&0.0);
            current += delta as i64;
        }

        self.v
            .push(NaiveDateTime::from_timestamp_opt(current / 1000, 0).unwrap());

        GeneratorState::Yielded(*self.v.last().unwrap())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use chrono::{Duration, NaiveDateTime};

    fn ndt(str: &str) -> NaiveDateTime {
        let fmt = "%Y-%m-%d %H:%M:%S";
        chrono::naive::NaiveDateTime::parse_from_str(str, fmt).unwrap()
    }

    #[test]
    fn test_incrementing_int() {
        let mut int_series = IncrementingSeries {
            current: 5,
            increment: 10,
        };

        let mut rng = rand::thread_rng();
        assert_eq!(5, int_series.next(&mut rng).into_yielded().unwrap());
        assert_eq!(15, int_series.next(&mut rng).into_yielded().unwrap());
        assert_eq!(25, int_series.next(&mut rng).into_yielded().unwrap())
    }

    #[test]
    fn test_incrementing_chrono() {
        let mut chrono_series = IncrementingSeries {
            current: ndt("2000-01-01 15:15:15"),
            increment: Duration::hours(1),
        };

        let mut rng = rand::thread_rng();
        assert_eq!(
            ndt("2000-01-01 15:15:15"),
            chrono_series.next(&mut rng).into_yielded().unwrap()
        );
        assert_eq!(
            ndt("2000-01-01 16:15:15"),
            chrono_series.next(&mut rng).into_yielded().unwrap()
        );
        assert_eq!(
            ndt("2000-01-01 17:15:15"),
            chrono_series.next(&mut rng).into_yielded().unwrap()
        );
    }

    #[test]
    fn test_poisson() {
        let initial = ndt("2000-01-01 15:15:15");
        let mut poisson = PoissonSeries {
            current: initial,
            rate: Duration::days(365),
        };

        let mut rng = rand::thread_rng();

        let iter1 = poisson.next(&mut rng).into_yielded().unwrap();
        let iter2 = poisson.next(&mut rng).into_yielded().unwrap();
        let iter3 = poisson.next(&mut rng).into_yielded().unwrap();
        assert!(initial < iter1);
        assert!(iter1 < iter2);
        assert!(iter2 < iter3);
    }

    #[test]
    fn test_cyclical() {
        let initial = ndt("2000-01-01 00:00:00");
        let mut cyclical = CyclicalSeries {
            start: initial,
            current: initial,
            period: Duration::weeks(1),
            min_rate: Duration::minutes(10),
            max_rate: Duration::hours(2),
        };

        let mut rng = rand::thread_rng();

        let iter1 = cyclical.next(&mut rng).into_yielded().unwrap();
        let iter2 = cyclical.next(&mut rng).into_yielded().unwrap();
        let iter3 = cyclical.next(&mut rng).into_yielded().unwrap();
        assert!(initial < iter1);
        assert!(iter1 < iter2);
        assert!(iter2 < iter3);
    }

    #[test]
    fn test_zip() {
        let incrementing_series1 = IncrementingSeries {
            current: ndt("2000-01-01 15:15:15"),
            increment: Duration::minutes(10),
        };

        let incrementing_series2 = IncrementingSeries {
            current: ndt("2000-01-01 15:15:30"),
            increment: Duration::minutes(15),
        };

        let mut zip_series = ZipSeries::new(vec![
            TimeSeries::Incrementing(incrementing_series1),
            TimeSeries::Incrementing(incrementing_series2),
        ])
        .unwrap();

        let mut rng = rand::thread_rng();

        assert_eq!(
            ndt("2000-01-01 15:15:15"),
            zip_series.next(&mut rng).into_yielded().unwrap()
        );
        assert_eq!(
            ndt("2000-01-01 15:15:30"),
            zip_series.next(&mut rng).into_yielded().unwrap()
        );
        assert_eq!(
            ndt("2000-01-01 15:25:15"),
            zip_series.next(&mut rng).into_yielded().unwrap()
        );
        assert_eq!(
            ndt("2000-01-01 15:30:30"),
            zip_series.next(&mut rng).into_yielded().unwrap()
        );
        assert_eq!(
            ndt("2000-01-01 15:35:15"),
            zip_series.next(&mut rng).into_yielded().unwrap()
        );
    }
}
