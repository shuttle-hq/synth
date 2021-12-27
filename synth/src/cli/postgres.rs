use crate::cli::export::{create_and_insert_values, ExportParams, ExportStrategy};
use crate::cli::import::ImportStrategy;
use crate::cli::import_utils::build_namespace_import;
use crate::datasource::postgres_datasource::{PostgresConnectParams, PostgresDataSource};
use crate::datasource::DataSource;
use crate::sampler::SamplerOutput;
use anyhow::Result;

#[derive(Clone, Debug)]
pub struct PostgresExportStrategy {
    pub uri_string: String,
    pub schema: Option<String>,
}

impl ExportStrategy for PostgresExportStrategy {
    fn export(&self, params: ExportParams) -> Result<SamplerOutput> {
        let connect_params = PostgresConnectParams {
            uri: self.uri_string.clone(),
            schema: self.schema.clone(),
        };

        let datasource = PostgresDataSource::new(&connect_params)?;

        create_and_insert_values(params, &datasource)
    }
}

#[derive(Clone, Debug)]
pub struct PostgresImportStrategy {
    pub uri_string: String,
    pub schema: Option<String>,
}

impl ImportStrategy for PostgresImportStrategy {
    fn import(&self) -> Result<Namespace> {
        let connect_params = PostgresConnectParams {
            uri: self.uri_string.clone(),
            schema: self.schema.clone(),
        };

        let datasource = PostgresDataSource::new(&connect_params)?;

        build_namespace_import(&datasource)
    }
}
