use crate::cli::export::{ExportParams, ExportStrategy};
use crate::cli::import::ImportStrategy;
use crate::sampler::{Sampler, SamplerOutput};

use synth_core::schema::{MergeStrategy, OptionalMergeStrategy};
use synth_core::Content;

use anyhow::{Context, Result};

use std::cell::RefCell;
use std::convert::TryFrom;
use std::io::Write;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct JsonFileExportStrategy {
    pub from_file: PathBuf,
}

impl ExportStrategy for JsonFileExportStrategy {
    fn export(&self, params: ExportParams) -> Result<SamplerOutput> {
        let generator = Sampler::try_from(&params.namespace)?;
        let output = generator.sample_seeded(params.collection_name, params.target, params.seed)?;

        std::fs::write(&self.from_file, output.clone().into_json().to_string())?;

        Ok(output)
    }
}

#[derive(Clone, Debug)]
pub struct JsonStdoutExportStrategy<W> {
    pub writer: RefCell<W>,
}

impl<W: Write> ExportStrategy for JsonStdoutExportStrategy<W> {
    fn export(&self, params: ExportParams) -> Result<SamplerOutput> {
        let generator = Sampler::try_from(&params.namespace)?;
        let output = generator.sample_seeded(params.collection_name, params.target, params.seed)?;

        writeln!(self.writer.borrow_mut(), "{}", output.clone().into_json())
            .expect("failed to write json output");

        Ok(output)
    }
}

#[derive(Clone, Debug)]
pub struct JsonFileImportStrategy {
    pub from_file: PathBuf,
}

impl ImportStrategy for JsonFileImportStrategy {
    fn import_namespace(&self) -> Result<Content> {
        import_json(serde_json::from_reader(std::fs::File::open(
            &self.from_file,
        )?)?)
    }
}

#[derive(Clone, Debug)]
pub struct JsonStdinImportStrategy;

impl ImportStrategy for JsonStdinImportStrategy {
    fn import_namespace(&self) -> Result<Content> {
        import_json(serde_json::from_reader(std::io::stdin())?)
    }
}

pub fn import_json(val: serde_json::Value) -> Result<Content> {
    match val {
        serde_json::Value::Object(object) => object
            .into_iter()
            .map(|(name, value)| {
                collection_from_value(&value)
                    .map(|content| (name.clone(), content))
                    .with_context(|| anyhow!("While importing the collection `{}`", name))
            })
            .collect(),
        unacceptable => Err(anyhow!(
            "Was expecting an object, instead got `{}`",
            unacceptable
        )),
    }
}

fn collection_from_value(value: &serde_json::Value) -> Result<Content> {
    match value {
        serde_json::Value::Array(values) => {
            let fst = values.first().unwrap_or(&serde_json::Value::Null);
            let mut as_content = Content::new_collection(fst.into());
            OptionalMergeStrategy.try_merge(&mut as_content, value)?;
            Ok(as_content)
        }
        unacceptable => Err(anyhow!(
            "Was expecting a collection, instead got `{}`",
            unacceptable
        )),
    }
}
