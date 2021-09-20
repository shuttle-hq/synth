use crate::cli::mongo::MongoImportStrategy;
use crate::cli::mysql::MySqlImportStrategy;
use crate::cli::postgres::PostgresImportStrategy;
use crate::cli::stdf::{FileImportStrategy, StdinImportStrategy};
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::PathBuf;
use std::str::FromStr;
use synth_core::graph::prelude::{MergeStrategy, OptionalMergeStrategy};
use synth_core::schema::Namespace;
use synth_core::{Content, Name};
use std::convert::TryFrom;

pub trait ImportStrategy {
    fn import(&self) -> Result<Namespace> {
        ns_from_value(self.into_value()?)
    }
    fn import_collection(&self, _name: &Name) -> Result<Content> {
        collection_from_value(&self.into_value()?)
    }
    fn into_value(&self) -> Result<Value>;
}

pub struct ImportParams {
    pub import_strategy: Option<String>,
    pub schema: Option<String>
}

impl TryFrom<ImportParams> for Box<dyn ImportStrategy> {
    type Error = anyhow::Error;

    fn try_from(value: ImportParams) -> Result<Self, Self::Error> {
        match value.import_strategy {
            None => Ok(Box::new(StdinImportStrategy {})),
            Some(uri) => {
                let import_strategy: Box<dyn ImportStrategy> = if uri.starts_with("postgres://") || uri.starts_with("postgresql://") {
                    Box::new(PostgresImportStrategy {
                        uri,
                        schema: value.schema
                    })
                } else if uri.starts_with("mongodb://") {
                    Box::new(MongoImportStrategy {
                        uri,
                    })
                } else if uri.starts_with("mysql://") || uri.starts_with("mariadb://") {
                    Box::new(MySqlImportStrategy {
                        uri,
                    })
                } else if let Ok(path) = PathBuf::from_str(&uri) {
                    Box::new(FileImportStrategy {
                        from_file: path,
                    })
                } else {
                    return Err(anyhow!(
                         "Data source not recognized. Was expecting one of 'mongodb' or 'postgres'"
                    ))
                };
                Ok(import_strategy)
            }
        }
    }
}

fn collection_from_value(value: &Value) -> Result<Content> {
    match value {
        Value::Array(values) => {
            let fst = values.get(0).unwrap_or(&Value::Null);
            let mut as_content = Namespace::collection(fst);
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
                    .with_context(|| anyhow!("While importing the collection `{}`", name))
            })
            .collect(),
        unacceptable => Err(anyhow!(
            "Was expecting an object, instead got `{}`",
            unacceptable
        )),
    }
}
