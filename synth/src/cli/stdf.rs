use crate::cli::export::{ExportParams, ExportStrategy};
use crate::cli::import::ImportStrategy;
use crate::sampler::Sampler;
use anyhow::Result;
use serde_json::Value;
use synth_core::{Content, Name};

use std::convert::TryFrom;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum DataFormat {
    Json,
    JsonLines {
        collection_field_name: Option<String>,
    },
    Csv,
}

impl DataFormat {
    pub fn new(format_string: Option<String>, collection_field_name: Option<String>) -> Self {
        format_string
            .map(|format_string| match format_string.as_str() {
                "jsonl" => DataFormat::JsonLines {
                    collection_field_name,
                },
                "csv" => DataFormat::Csv,
                _ => DataFormat::Json,
            })
            .unwrap_or_default()
    }
}

impl Default for DataFormat {
    fn default() -> Self {
        DataFormat::Json
    }
}

#[derive(Clone, Debug)]
pub struct FileImportStrategy {
    pub from_file: PathBuf,
    pub data_format: DataFormat,
}

#[derive(Clone, Debug)]
pub struct StdinImportStrategy {
    pub data_format: DataFormat,
}

#[derive(Clone, Debug)]
pub struct StdoutExportStrategy {
    pub data_format: DataFormat,
}

impl ExportStrategy for StdoutExportStrategy {
    fn export(&self, params: ExportParams) -> Result<()> {
        let generator = Sampler::try_from(&params.namespace)?;
        let output = generator.sample_seeded(params.collection_name, params.target, params.seed)?;

        match &self.data_format {
            DataFormat::Json => println!("{}", output.into_json()),
            DataFormat::JsonLines {
                collection_field_name,
            } => {
                let field_name = collection_field_name.as_deref().unwrap_or("collection"); // Default collection field name is 'collection'.

                // TODO: Warn user if the collection field name would overwrite an existing field in a collection.

                for line_val in output.into_json_lines(field_name) {
                    println!("{}", line_val);
                }
            }
            DataFormat::Csv => unimplemented!(),
        }

        Ok(())
    }
}

impl ImportStrategy for FileImportStrategy {
    fn import_collection(&self, name: &Name) -> Result<Content> {
        self.import()?
            .collections
            .remove(name)
            .ok_or_else(|| anyhow!("Could not find collection '{}' in file.", name))
    }

    fn as_value(&self) -> Result<Value> {
        Ok(serde_json::from_reader(std::fs::File::open(
            self.from_file.clone(),
        )?)?)
    }
}

impl ImportStrategy for StdinImportStrategy {
    fn as_value(&self) -> Result<Value> {
        Ok(serde_json::from_reader(std::io::stdin())?)
    }
}
