use crate::cli::postgres::PostgresExportStrategy;
use crate::cli::stdf::StdoutExportStrategy;
use anyhow::{Context, Result};

use std::str::FromStr;

use crate::cli::mongo::MongoExportStrategy;
use crate::cli::mysql::MySqlExportStrategy;
use crate::datasource::DataSource;
use crate::sampler::Sampler;
use async_std::task;
use serde_json::Value;
use synth_core::{Name, Namespace};

pub trait ExportStrategy {
    fn export(self, params: ExportParams) -> Result<()>;
}

pub struct ExportParams {
    pub namespace: Namespace,
    pub collection_name: Option<Name>,
    pub target: usize,
    pub seed: u64,
}

#[derive(Clone, Debug)]
pub enum SomeExportStrategy {
    StdoutExportStrategy(StdoutExportStrategy),
    FromPostgres(PostgresExportStrategy),
    FromMongo(MongoExportStrategy),
    FromMySql(MySqlExportStrategy),
}

impl ExportStrategy for SomeExportStrategy {
    fn export(self, params: ExportParams) -> Result<()> {
        match self {
            SomeExportStrategy::StdoutExportStrategy(ses) => ses.export(params),
            SomeExportStrategy::FromPostgres(pes) => pes.export(params),
            SomeExportStrategy::FromMongo(mes) => mes.export(params),
            SomeExportStrategy::FromMySql(mes) => mes.export(params),
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
        } else if s.starts_with("mysql://") || s.starts_with("mariadb://") {
            return Ok(SomeExportStrategy::FromMySql(MySqlExportStrategy {
                uri: s.to_string(),
            }));
        }
        Err(anyhow!(
            "Data sink not recognized. Was expecting one of 'mongodb' or 'postgres'"
        ))
    }
}

pub(crate) fn create_and_insert_values<T: DataSource>(
    params: ExportParams,
    datasource: &T,
) -> Result<()> {
    let sampler = Sampler::new(&params.namespace);
    let values =
        sampler.sample_seeded(params.collection_name.clone(), params.target, params.seed)?;

    match values {
        Value::Array(collection_json) => {
            insert_data(datasource, params.collection_name.unwrap().to_string(), &collection_json)
        }
        Value::Object(namespace_json) => {
            let names = params.namespace.topo_sort().ok_or(anyhow::anyhow!("dependency is cyclic"))?;
            for n in names.iter() {
                let collection_json = namespace_json.get(&n.to_string())
                    .expect("did not find Value for name")
                    .as_array()
                    .expect("This is always a collection (sampler contract)");
                insert_data(datasource, n.to_string().clone(), &collection_json)?;
            };
            Ok(())
        }
        _ => bail!(
            "The sampler will never generate a value which is not an array or object (sampler contract)"
        ),
    }
}

fn insert_data<T: DataSource>(datasource: &T, collection_name: String, collection_json: &[Value])
    -> Result<()> {
    task::block_on(
        datasource.insert_data(
            collection_name.clone(),
            collection_json
        )
    ).with_context(|| format!("Failed to insert data for collection {}", collection_name))
}
