use crate::cli::mongo::MongoExportStrategy;
use crate::cli::mysql::MySqlExportStrategy;
use crate::cli::postgres::PostgresExportStrategy;
use crate::cli::stdf::StdoutExportStrategy;

use anyhow::{Context, Result};

use std::convert::TryFrom;

use crate::cli::db_utils::DataSourceParams;
use crate::datasource::DataSource;
use crate::sampler::{Sampler, SamplerOutput};
use async_std::task;
use synth_core::{Name, Namespace, Value};

use super::DataFormat;

pub trait ExportStrategy {
    fn export(&self, params: ExportParams) -> Result<()>;
}

pub struct ExportParams {
    pub namespace: Namespace,
    /// The name of the single collection to generate from if one is specified (via --collection).
    pub collection_name: Option<Name>,
    pub target: usize,
    pub seed: u64,
}

impl TryFrom<DataSourceParams<'_>> for Box<dyn ExportStrategy> {
    type Error = anyhow::Error;

    fn try_from(params: DataSourceParams) -> Result<Self, Self::Error> {
        // Due to all the schemas used, with the exception of 'mongodb', being non-standard (including
        // 'postgres' and 'mysql' suprisingly) it seems simpler to just match based on the scheme string
        // instead of on enum variants.
        let scheme = params.uri.scheme().as_str().to_lowercase();
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
            "json" | "jsonl" | "csv" => {
                let data_format = DataFormat::new(&scheme, params.collection_field_name);

                if params.uri.path() == "" {
                    Box::new(StdoutExportStrategy { data_format })
                } else {
                    unimplemented!();
                    // TODO: File exporting!
                    /*Box::new(FileExportStrategy {
                        data_format,
                        to_file: PathBuf::from(params.uri.path().to_string()),
                    })*/
                }
            }
            _ => {
                return Err(anyhow!(
                    "Data sink not recognized. Was expecting one of 'mongodb', 'postgres', 'mysql' or 'mariadb'."
                ));
            }
        };
        Ok(export_strategy)
    }
}

pub(crate) fn create_and_insert_values<T: DataSource>(
    params: ExportParams,
    datasource: &T,
) -> Result<()> {
    let sampler = Sampler::try_from(&params.namespace)?;
    let values =
        sampler.sample_seeded(params.collection_name.clone(), params.target, params.seed)?;

    match values {
        SamplerOutput::Collection(collection) => insert_data(
            datasource,
            params.collection_name.unwrap().to_string(),
            &collection,
        ),
        SamplerOutput::Namespace(namespace) => {
            for (name, collection) in namespace {
                insert_data(datasource, name, &collection)?;
            }
            Ok(())
        }
    }
}

fn insert_data<T: DataSource>(
    datasource: &T,
    collection_name: String,
    collection: &[Value],
) -> Result<()> {
    task::block_on(datasource.insert_data(collection_name.clone(), collection))
        .with_context(|| format!("Failed to insert data for collection {}", collection_name))
}
