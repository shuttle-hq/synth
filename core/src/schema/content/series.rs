use super::prelude::*;
use crate::graph::series::{
    CyclicalSeries, IncrementingSeries, PoissonSeries, TimeSeries, ZipSeries,
};
use crate::{Compile, Compiler, Graph};
use anyhow::Result;
use std::convert::TryInto;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SeriesContent {
    pub format: Option<String>,
    #[serde(flatten)]
    pub variant: SeriesVariant,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum SeriesVariant {
    Incrementing(Incrementing),
    Poisson(Poisson),
    Cyclical(Cyclical),
    Zip(Zip),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Incrementing {
    pub(crate) start: String,
    #[serde(with = "humantime_serde")]
    pub(crate) increment: std::time::Duration,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Poisson {
    pub(crate) start: String,
    #[serde(with = "humantime_serde")]
    pub(crate) rate: std::time::Duration,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Cyclical {
    pub(crate) start: String,
    #[serde(with = "humantime_serde")]
    pub(crate) period: std::time::Duration,
    #[serde(with = "humantime_serde")]
    pub(crate) min_rate: std::time::Duration,
    #[serde(with = "humantime_serde")]
    pub(crate) max_rate: std::time::Duration,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Zip {
    series: Vec<SeriesVariant>,
}

impl Compile for SeriesContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, _compiler: C) -> Result<Graph> {
        Ok(Graph::Series(self.try_into()?))
    }
}

impl SeriesVariant {
    pub(crate) fn inc(
        incrementing: &Incrementing,
        fmt: &String,
    ) -> Result<IncrementingSeries<NaiveDateTime, Duration>> {
        Ok(IncrementingSeries::new(
            NaiveDateTime::parse_from_str(&incrementing.start, fmt)?,
            Duration::from_std(incrementing.increment.into())?,
        ))
    }

    pub(crate) fn poisson(poisson: &Poisson, fmt: &String) -> Result<PoissonSeries> {
        Ok(PoissonSeries::new(
            NaiveDateTime::parse_from_str(&poisson.start, fmt)?,
            Duration::from_std(poisson.rate.into())?,
        ))
    }

    pub(crate) fn cyclical(cyclical: &Cyclical, fmt: &String) -> Result<CyclicalSeries> {
        Ok(CyclicalSeries::new(
            NaiveDateTime::parse_from_str(&cyclical.start, fmt)?,
            Duration::from_std(cyclical.period.into())?,
            Duration::from_std(cyclical.min_rate.into())?,
            Duration::from_std(cyclical.max_rate.into())?,
        ))
    }

    pub(crate) fn zip(zip: &Zip, fmt: &String) -> Result<ZipSeries> {
        let mut children = Vec::new();
        for child in &zip.series {
            let series = match child {
                SeriesVariant::Incrementing(inc) => TimeSeries::Incrementing(Self::inc(&inc, fmt)?),
                SeriesVariant::Poisson(poi) => TimeSeries::Poisson(Self::poisson(&poi, fmt)?),
                SeriesVariant::Cyclical(cyc) => TimeSeries::Cyclical(Self::cyclical(&cyc, fmt)?),
                SeriesVariant::Zip(zip) => TimeSeries::Zip(Self::zip(&zip, fmt)?),
            };
            children.push(series);
        }
        ZipSeries::new(children)
    }
}
