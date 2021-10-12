use crate::cli::export::{ExportParams, ExportStrategy};
use crate::sampler::Sampler;
use anyhow::Result;
use std::convert::TryFrom;

#[derive(Clone, Debug)]
pub struct JsonLinesExportStrategy {}

impl ExportStrategy for JsonLinesExportStrategy {
    fn export(&self, params: ExportParams) -> Result<()> {
        let generator = Sampler::try_from(&params.namespace)?;
        let output = generator.sample_seeded(params.collection_name, params.target, params.seed)?;

        for line_val in output.into_json_lines() {
            println!("{}", line_val);
        }

        Ok(())
    }
}
