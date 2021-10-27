use crate::cli::export::{ExportParams, ExportStrategy};
use crate::cli::import::ImportStrategy;
use crate::sampler::{CsvOutput, Sampler};
use anyhow::Result;
use serde_json::Value;

use super::DataFormat;
use std::convert::TryFrom;
use std::io::BufRead;
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
    fn export(&self, params: ExportParams) -> Result<()> {
        let generator = Sampler::try_from(&params.namespace)?;
        let output = generator.sample_seeded(params.collection_name, params.target, params.seed)?;

        match &self.data_format {
            DataFormat::Json => println!("{}", output.into_json()),
            DataFormat::JsonLines { .. } => {
                // TODO: Warn user if the collection field name would overwrite an existing field in a collection.

                for line in
                    output.into_json_lines(self.data_format.get_collection_field_name_or_default())
                {
                    println!("{}", line);
                }
            }
            DataFormat::Csv => match output.into_csv(&params.namespace)? {
                CsvOutput::Namespace(ns) => {
                    for (name, csv) in ns {
                        println!("\n{}\n{}\n\n{}\n", name, "-".repeat(name.len()), csv)
                    }
                }
                CsvOutput::SingleCollection(csv) => println!("{}", csv),
            },
        }

        Ok(())
    }
}
