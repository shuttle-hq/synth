use crate::cli::export::{create_and_insert_values, ExportParams, ExportStrategy};
use crate::cli::import::ImportStrategy;
use crate::cli::import_utils::build_namespace_import;
use crate::datasource::mysql_datasource::MySqlDataSource;
use crate::datasource::DataSource;
use crate::sampler::SamplerOutput;
use anyhow::Result;
use synth_core::schema::Namespace;
use synth_core::{Content, Name};

#[derive(Clone, Debug)]
pub struct MySqlExportStrategy {
    pub uri_string: String,
}

impl ExportStrategy for MySqlExportStrategy {
    fn export(&self, params: ExportParams) -> Result<SamplerOutput> {
        let datasource = MySqlDataSource::new(&self.uri_string)?;

        create_and_insert_values(params, &datasource)
    }
}

#[derive(Clone, Debug)]
pub struct MySqlImportStrategy {
    pub uri_string: String,
}

impl ImportStrategy for MySqlImportStrategy {
    fn import(&self) -> Result<Namespace> {
        let datasource = MySqlDataSource::new(&self.uri_string)?;

        build_namespace_import(&datasource)
    }

    fn import_collection(&self, name: &Name) -> Result<Content> {
        self.import()?
            .collections
            .remove(name)
            .ok_or_else(|| anyhow!("Could not find table '{}' in Postgres database.", name))
    }
}
