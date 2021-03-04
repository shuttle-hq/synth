use crate::cli::export::{ExportParams, ExportStrategy};
use crate::cli::import::ImportStrategy;
use crate::sampler::Sampler;
use anyhow::Result;
use serde_json::Value;

use std::convert::TryFrom;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub(crate) struct FileImportStrategy {
    pub(crate) from_file: PathBuf,
}

#[derive(Clone, Debug)]
pub(crate) struct StdinImportStrategy {}

#[derive(Clone, Debug)]
pub(crate) struct StdoutExportStrategy {}

impl ExportStrategy for StdoutExportStrategy {
    fn export(self, params: ExportParams) -> Result<()> {
        let generator = Sampler::try_from(&params.namespace)?;
        let values = generator.sample(params.collection_name, params.target)?;
        println!("{}", values);
        Ok(())
    }
}

impl ImportStrategy for FileImportStrategy {
    fn into_value(self) -> Result<Value> {
        Ok(serde_json::from_reader(std::fs::File::open(
            self.from_file,
        )?)?)
    }
}

impl ImportStrategy for StdinImportStrategy {
    fn into_value(self) -> Result<Value> {
        Ok(serde_json::from_reader(std::io::stdin())?)
    }
}
