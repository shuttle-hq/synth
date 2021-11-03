use crate::cli::export::{ExportParams, ExportStrategy};
use crate::cli::import::ImportStrategy;
use crate::sampler::{Sampler, SamplerOutput};
use anyhow::Result;
use serde_json::Value;

use super::DataFormat;
use std::convert::TryFrom;
use std::io::{BufRead, Write};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct FileImportStrategy {
    pub from_file: PathBuf,
    pub data_format: DataFormat,
}

impl ImportStrategy for FileImportStrategy {
    fn get_data_format(&self) -> &DataFormat {
        &self.data_format
    }

    fn as_json_value(&self) -> Result<Value> {
        Ok(serde_json::from_reader(std::fs::File::open(
            &self.from_file,
        )?)?)
    }

    fn as_json_line_values(&self) -> Result<Vec<Value>> {
        Ok(
            std::io::BufReader::new(std::fs::File::open(&self.from_file)?)
                .lines()
                .map(|line| serde_json::from_str(&line.unwrap()))
                .collect::<serde_json::Result<Vec<Value>>>()?,
        )
    }
}

#[derive(Clone, Debug)]
pub struct FileExportStrategy {
    pub from_file: PathBuf,
    pub data_format: DataFormat,
}

impl ExportStrategy for FileExportStrategy {
    fn export(&self, params: ExportParams) -> Result<SamplerOutput> {
        let generator = Sampler::try_from(&params.namespace)?;
        let output = generator.sample_seeded(params.collection_name, params.target, params.seed)?;

        match &self.data_format {
            DataFormat::Json => {
                std::fs::write(&self.from_file, output.clone().into_json().to_string())?
            }
            DataFormat::JsonLines {
                collection_field_name,
            } => {
                let mut f = std::io::BufWriter::new(std::fs::File::create(&self.from_file)?);

                for val in output.clone().into_json_lines(collection_field_name) {
                    f.write_all((val.to_string() + "\n").as_bytes())?;
                }
            }
        }

        Ok(output)
    }
}

#[derive(Clone, Debug)]
pub struct StdinImportStrategy {
    pub data_format: DataFormat,
}

impl ImportStrategy for StdinImportStrategy {
    fn get_data_format(&self) -> &DataFormat {
        &self.data_format
    }

    fn as_json_value(&self) -> Result<Value> {
        Ok(serde_json::from_reader(std::io::stdin())?)
    }

    fn as_json_line_values(&self) -> Result<Vec<Value>> {
        Ok(std::io::stdin()
            .lock()
            .lines()
            .map(|line| serde_json::from_str(&line.unwrap()))
            .collect::<serde_json::Result<Vec<Value>>>()?)
    }
}

#[derive(Clone, Debug)]
pub struct StdoutExportStrategy {
    pub data_format: DataFormat,
}

impl ExportStrategy for StdoutExportStrategy {
    fn export(&self, params: ExportParams) -> Result<SamplerOutput> {
        let generator = Sampler::try_from(&params.namespace)?;
        let output = generator.sample_seeded(params.collection_name, params.target, params.seed)?;

        match &self.data_format {
            DataFormat::Json => println!("{}", output.clone().into_json()),
            DataFormat::JsonLines {
                collection_field_name,
            } => {
                // TODO: Warn user if the collection field name would overwrite an existing field in a collection.

                for line in output.clone().into_json_lines(collection_field_name) {
                    println!("{}", line);
                }
            }
        }

        Ok(output)
    }
}
