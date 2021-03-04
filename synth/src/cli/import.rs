use crate::cli::postgres::PostgresImportStrategy;
use crate::cli::stdf::{FileImportStrategy, StdinImportStrategy};
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::PathBuf;
use std::str::FromStr;
use synth_core::graph::prelude::{MergeStrategy, OptionalMergeStrategy};
use synth_core::schema::Namespace;
use synth_core::{Content, Name};

pub(crate) trait ImportStrategy: Sized {
    fn import(self) -> Result<Namespace> {
        ns_from_value(self.into_value()?)
    }
    fn import_collection(self, _name: &Name) -> Result<Content> {
        collection_from_value(&self.into_value()?)
    }
    fn into_value(self) -> Result<Value>;
}

#[derive(Clone, Debug)]
pub(crate) enum SomeImportStrategy {
    StdinImportStrategy(StdinImportStrategy),
    FromFile(FileImportStrategy),
    #[allow(unused)]
    FromPostgres(PostgresImportStrategy),
}

impl Default for SomeImportStrategy {
    fn default() -> Self {
        SomeImportStrategy::StdinImportStrategy(StdinImportStrategy {})
    }
}

impl FromStr for SomeImportStrategy {
    type Err = anyhow::Error;

    /// Here we exhaustively try to pattern match strings until we get something
    /// that successfully parses. Starting from a file, could be a url to a database etc.
    /// We assume that these can be unambiguously identified for now.
    /// For example, `postgres://...` is not going to be a file on the FS
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // for postgres, `postgres` or `postgresql` are fine
        if s.starts_with("postgres") {
            return Ok(SomeImportStrategy::FromPostgres(PostgresImportStrategy {
                uri: s.to_string(),
            }));
        }

        if let Ok(path) = PathBuf::from_str(s) {
            return Ok(SomeImportStrategy::FromFile(FileImportStrategy {
                from_file: path,
            }));
        }
        Err(anyhow!("Data source not recognized"))
    }
}

impl ImportStrategy for SomeImportStrategy {
    fn import(self) -> Result<Namespace> {
        match self {
            SomeImportStrategy::FromFile(fis) => fis.import(),
            SomeImportStrategy::FromPostgres(pis) => pis.import(),
            SomeImportStrategy::StdinImportStrategy(sis) => sis.import(),
        }
    }
    fn import_collection(self, name: &Name) -> Result<Content> {
        match self {
            SomeImportStrategy::FromFile(fis) => fis.import_collection(name),
            SomeImportStrategy::FromPostgres(pis) => pis.import_collection(name),
            SomeImportStrategy::StdinImportStrategy(sis) => sis.import_collection(name),
        }
    }
    fn into_value(self) -> Result<Value> {
        match self {
            SomeImportStrategy::FromFile(fis) => fis.into_value(),
            SomeImportStrategy::FromPostgres(pis) => pis.into_value(),
            SomeImportStrategy::StdinImportStrategy(sis) => sis.into_value(),
        }
    }
}

fn collection_from_value(value: &Value) -> Result<Content> {
    match value {
        Value::Array(values) => {
            let fst = values.get(0).unwrap_or(&Value::Null);
            let mut as_content = Namespace::collection(&fst);
            OptionalMergeStrategy.try_merge(&mut as_content, value)?;
            Ok(as_content)
        }
        unacceptable => Err(anyhow!(
            "Was expecting a collection, instead got `{}`",
            unacceptable
        )),
    }
}

fn ns_from_value(value: Value) -> Result<Namespace> {
    match value {
        Value::Object(object) => object
            .into_iter()
            .map(|(name, value)| {
                collection_from_value(&value)
                    .and_then(|content| Ok((name.parse()?, content)))
                    .context(anyhow!("While importing the collection `{}`", name))
            })
            .collect(),
        unacceptable => Err(anyhow!(
            "Was expecting an object, instead got `{}`",
            unacceptable
        )),
    }
}
