use crate::cli::export::{create_and_insert_values, ExportParams, ExportStrategy};
use crate::cli::import::ImportStrategy;
use crate::cli::import_utils::build_namespace_import;
use crate::datasource::postgres_datasource::{PostgresConnectParams, PostgresDataSource};
use crate::datasource::DataSource;
use anyhow::Result;
use serde_json::Value;
use synth_core::schema::Namespace;
use synth_core::{Content, Name};

#[derive(Clone, Debug)]
pub struct PostgresExportStrategy {
    pub uri: String,
    pub schema: Option<String>,
}

impl ExportStrategy for PostgresExportStrategy {
    fn export(&self, params: ExportParams) -> Result<()> {
        let connect_params = PostgresConnectParams {
            uri: self.uri.clone(),
            schema: self.schema.clone(),
        };

        let datasource = PostgresDataSource::new(&connect_params)?;

        create_and_insert_values(params, &datasource)
    }
}

#[derive(Clone, Debug)]
pub struct PostgresImportStrategy {
    pub uri: String,
    pub schema: Option<String>,
}

impl ImportStrategy for PostgresImportStrategy {
    fn import(&self) -> Result<Namespace> {
        let connect_params = PostgresConnectParams {
            uri: self.uri.clone(),
            schema: self.schema.clone(),
        };

        let datasource = PostgresDataSource::new(&connect_params)?;

        build_namespace_import(&datasource)
    }

    fn import_collection(&self, name: &Name) -> Result<Content> {
        self.import()?
            .collections
            .remove(name)
            .ok_or_else(|| anyhow!("Could not find table '{}' in Postgres database.", name))
    }

    fn as_value(&self) -> Result<Value> {
        bail!("Postgres import doesn't support conversion into value")
    }
}
