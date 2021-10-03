use crate::cli::export::{create_and_insert_values, ExportParams, ExportStrategy};
use crate::cli::import::ImportStrategy;
use crate::cli::import_utils::build_namespace_import;
use crate::datasource::sqlite_datasource::SqliteDataSource;
use crate::datasource::DataSource;
use anyhow::Result;
use serde_json::Value;
use synth_core::schema::Namespace;
use synth_core::{Content, Name};

#[derive(Clone, Debug)]
pub struct SqliteExportStrategy {
    pub uri: String,
}

impl ExportStrategy for SqliteExportStrategy {
    fn export(&self, params: ExportParams) -> Result<()> {
        let datasource = SqliteDataSource::new(&self.uri)?;

        create_and_insert_values(params, &datasource)
    }
}

#[derive(Clone, Debug)]
pub struct SqliteImportStrategy {
    pub uri: String,
}

impl ImportStrategy for SqliteImportStrategy {
    fn import(&self) -> Result<Namespace> {
        let datasource = SqliteDataSource::new(&self.uri)?;

        build_namespace_import(&datasource)
    }

    fn import_collection(&self, name: &Name) -> Result<Content> {
        self.import()?
            .collections
            .remove(name)
            .ok_or_else(|| anyhow!("Could not find table '{}' in Sqlite database.", name))
    }

    fn as_value(&self) -> Result<Value> {
        bail!("Sqlite import doesn't support conversion into value")
    }
}
