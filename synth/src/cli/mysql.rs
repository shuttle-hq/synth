use crate::cli::export::{ExportStrategy, ExportParams, create_and_insert_values};
use crate::datasource::mysql_datasource::MySqlDataSource;
use crate::datasource::DataSource;
use anyhow::Result;

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