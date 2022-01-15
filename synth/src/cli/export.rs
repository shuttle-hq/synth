use crate::cli::csv::{CsvFileExportStrategy, CsvStdoutExportStrategy};
use crate::cli::json::{JsonFileExportStrategy, JsonStdoutExportStrategy};
use crate::cli::jsonl::{JsonLinesFileExportStrategy, JsonLinesStdoutExportStrategy};
use crate::cli::mongo::MongoExportStrategy;
use crate::cli::mysql::MySqlExportStrategy;
use crate::cli::postgres::PostgresExportStrategy;

use anyhow::{Context, Result};

use std::convert::TryFrom;
use std::path::PathBuf;

use crate::datasource::DataSource;
use crate::sampler::{Sampler, SamplerOutput};
use async_std::task;
use synth_core::{DataSourceParams, Namespace, Value};

use super::map_from_uri_query;

pub(crate) trait ExportStrategy {
    fn export(&self, params: ExportParams) -> Result<SamplerOutput>;
}

pub struct ExportParams {
    pub namespace: Namespace,
    /// The name of the single collection to generate from if one is specified (via --collection).
    pub collection_name: Option<String>,
    pub target: usize,
    pub seed: u64,
    pub ns_path: PathBuf,
}

impl TryFrom<DataSourceParams<'_>> for Box<dyn ExportStrategy> {
    type Error = anyhow::Error;

    fn try_from(params: DataSourceParams) -> Result<Self, Self::Error> {
        // Due to all the schemes used, with the exception of 'mongodb', being non-standard (including 'postgres' and
        // 'mysql' suprisingly) it seems simpler to just match based on the scheme string instead of on enum variants.
        let scheme = params.uri.scheme().as_str().to_lowercase();
        let query = map_from_uri_query(params.uri.query());

        let export_strategy: Box<dyn ExportStrategy> = match scheme.as_str() {
            "postgres" | "postgresql" => Box::new(PostgresExportStrategy {
                uri_string: params.uri.to_string(),
                schema: params.schema,
            }),
            "mongodb" => Box::new(MongoExportStrategy {
                uri_string: params.uri.to_string(),
            }),
            "mysql" | "mariadb" => Box::new(MySqlExportStrategy {
                uri_string: params.uri.to_string(),
            }),
            "json" => {
                if params.uri.path() == "" {
                    Box::new(JsonStdoutExportStrategy)
                } else {
                    Box::new(JsonFileExportStrategy {
                        from_file: PathBuf::from(params.uri.path().to_string()),
                    })
                }
            }
            "jsonl" => {
                let collection_field_name = query
                    .get("collection_field_name")
                    .unwrap_or(&"type")
                    .to_string();

                if params.uri.path() == "" {
                    Box::new(JsonLinesStdoutExportStrategy {
                        collection_field_name,
                    })
                } else {
                    Box::new(JsonLinesFileExportStrategy {
                        from_file: PathBuf::from(params.uri.path().to_string()),
                        collection_field_name,
                    })
                }
            }
            "csv" => {
                if params.uri.path() == "" {
                    Box::new(CsvStdoutExportStrategy)
                } else {
                    Box::new(CsvFileExportStrategy {
                        to_dir: PathBuf::from(params.uri.path().to_string()),
                    })
                }
            }
            _ => {
                return Err(anyhow!(
                    "Export URI scheme not recognised. Was expecting one of 'mongodb', 'postgres', 'mysql', 'mariadb', 'json', 'jsonl' or 'csv'."
                ));
            }
        };
        Ok(export_strategy)
    }
}

pub(crate) fn create_and_insert_values<T: DataSource>(
    params: ExportParams,
    datasource: &T,
) -> Result<SamplerOutput> {
    let sampler = Sampler::try_from(&params.namespace)?;
    let sample =
        sampler.sample_seeded(params.collection_name.clone(), params.target, params.seed)?;

    match sample.clone() {
        SamplerOutput::Collection(name, value) => {
            insert_data(datasource, name.as_ref(), value)?;
        }
        SamplerOutput::Namespace(namespace) => {
            for (name, value) in namespace.into_iter() {
                insert_data(datasource, name.as_ref(), value)?;
            }
        }
    };

    Ok(sample)
}

fn insert_data<T: DataSource>(
    datasource: &T,
    collection_name: &str,
    collection: Value,
) -> Result<()> {
    let to_insert = match collection {
        Value::Array(elems) => elems,
        non_array => vec![non_array],
    };
    task::block_on(datasource.insert_data(collection_name, &to_insert[..]))
        .with_context(|| format!("Failed to insert data for collection {}", collection_name))
}
