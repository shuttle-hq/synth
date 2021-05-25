use crate::cli::postgres::PostgresExportStrategy;
use crate::cli::stdf::StdoutExportStrategy;
use anyhow::Result;

use std::str::FromStr;

use crate::cli::mongo::MongoExportStrategy;
use synth_core::{Name, Namespace};

pub(crate) trait ExportStrategy {
    fn export(self, params: ExportParams) -> Result<()>;
}

pub(crate) struct ExportParams {
    pub(crate) namespace: Namespace,
    pub(crate) collection_name: Option<Name>,
    pub(crate) target: usize,
    pub(crate) seed: u64,
}

#[derive(Clone, Debug)]
pub(crate) enum SomeExportStrategy {
    StdoutExportStrategy(StdoutExportStrategy),
    FromPostgres(PostgresExportStrategy),
    FromMongo(MongoExportStrategy),
}

impl ExportStrategy for SomeExportStrategy {
    fn export(self, params: ExportParams) -> Result<()> {
        match self {
            SomeExportStrategy::StdoutExportStrategy(ses) => ses.export(params),
            SomeExportStrategy::FromPostgres(pes) => pes.export(params),
            SomeExportStrategy::FromMongo(mes) => mes.export(params),
        }
    }
}

impl Default for SomeExportStrategy {
    fn default() -> Self {
        SomeExportStrategy::StdoutExportStrategy(StdoutExportStrategy {})
    }
}

impl FromStr for SomeExportStrategy {
    type Err = anyhow::Error;

    /// Here we exhaustively try to pattern match strings until we get something
    /// that successfully parses. Starting from a file, could be a url to a database etc.
    /// We assume that these can be unambiguously identified for now.
    /// For example, `postgres://...` is not going to be a file on the FS
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // for postgres, `postgres` or `postgresql` are fine
        if s.starts_with("postgres://") || s.starts_with("postgresql://") {
            return Ok(SomeExportStrategy::FromPostgres(PostgresExportStrategy {
                uri: s.to_string(),
            }));
        } else if s.starts_with("mongodb://") {
            return Ok(SomeExportStrategy::FromMongo(MongoExportStrategy {
                uri: s.to_string(),
            }));
        }
        Err(anyhow!(
            "Data sink not recognized. Was expecting one of 'mongodb' or 'postgres'"
        ))
    }
}
