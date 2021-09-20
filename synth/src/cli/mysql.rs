use crate::cli::export::{create_and_insert_values, ExportParams, ExportStrategy};
use crate::cli::import::ImportStrategy;
use crate::cli::import_utils::build_namespace_import;
use crate::datasource::mysql_datasource::MySqlDataSource;
use crate::datasource::DataSource;
use anyhow::Result;
use serde_json::Value;
use synth_core::schema::Namespace;
use synth_core::{Content, Name};

#[derive(Clone, Debug)]
pub struct MySqlExportStrategy {
    pub uri: String,
}

impl ExportStrategy for MySqlExportStrategy {
    fn export(self, params: ExportParams) -> Result<()> {
        let datasource = MySqlDataSource::new(&self.uri)?;

        create_and_insert_values(params, &datasource)
    }
}

#[derive(Clone, Debug)]
pub struct MySqlImportStrategy {
    pub uri: String,
}

impl ImportStrategy for MySqlImportStrategy {
    fn import(&self) -> Result<Namespace> {
        let datasource = MySqlDataSource::new(&self.uri)?;

        build_namespace_import(&datasource)
    }

    fn import_collection(&self, name: &Name) -> Result<Content> {
        self.import()?
            .collections
            .remove(name)
            .ok_or_else(|| anyhow!("Could not find table '{}' in Postgres database.", name))
    }

    fn into_value(&self) -> Result<Value> {
        bail!("MySql import doesn't support conversion into value")
    }
}
