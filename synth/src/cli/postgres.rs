use crate::cli::export::{ExportParams, ExportStrategy, create_and_insert_values};
use crate::cli::import::ImportStrategy;
use anyhow::Result;
use serde_json::Value;
use synth_core::schema::Namespace;
use synth_core::{Content, Name};
use crate::datasource::postgres_datasource::PostgresDataSource;
use crate::datasource::DataSource;
use crate::cli::import_utils::build_namespace_import;

#[derive(Clone, Debug)]
pub struct PostgresExportStrategy {
    pub uri: String,
}

impl ExportStrategy for PostgresExportStrategy {
    fn export(self, params: ExportParams) -> Result<()> {
        let datasource = PostgresDataSource::new(&self.uri)?;

        create_and_insert_values(params, &datasource)
    }
}

#[derive(Clone, Debug)]
pub struct PostgresImportStrategy {
    pub uri: String,
}

impl ImportStrategy for PostgresImportStrategy {
    fn import(self) -> Result<Namespace> {
        let datasource = PostgresDataSource::new(&self.uri)?;

        build_namespace_import(&datasource)
    }

    fn import_collection(self, name: &Name) -> Result<Content> {
        self.import()?
            .collections
            .remove(name)
            .ok_or_else(|| anyhow!("Could not find table '{}' in Postgres database.", name))
    }

    fn into_value(self) -> Result<Value> {
        unreachable!()
    }
}

