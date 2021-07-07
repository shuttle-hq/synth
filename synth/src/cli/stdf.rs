use crate::cli::export::{ExportParams, ExportStrategy};
use crate::cli::import::ImportStrategy;
use crate::sampler::Sampler;
use anyhow::Result;
use serde_json::Value;

use std::path::PathBuf;

use crate::cli::json::synth_val_to_json;

#[derive(Clone, Debug)]
pub struct FileImportStrategy {
    pub from_file: PathBuf,
}

#[derive(Clone, Debug)]
pub struct StdinImportStrategy {}

#[derive(Clone, Debug)]
pub struct StdoutExportStrategy {}

impl ExportStrategy for StdoutExportStrategy {
    fn export(self, params: ExportParams) -> Result<()> {
        let generator = Sampler::new(&params.namespace);
        let values = generator.sample_seeded(params.collection_name, params.target, params.seed)?;
        println!("{}", synth_val_to_json(values));
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
