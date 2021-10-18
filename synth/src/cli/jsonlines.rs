use crate::cli::export::{ExportParams, ExportStrategy};
use crate::sampler::Sampler;
use anyhow::Result;
use std::convert::TryFrom;

#[derive(Clone, Debug)]
pub struct JsonLinesExportStrategy {
    pub collection_field_name: Option<String>,
}

impl ExportStrategy for JsonLinesExportStrategy {
    fn export(&self, params: ExportParams) -> Result<()> {
        let generator = Sampler::try_from(&params.namespace)?;
        let output = generator.sample_seeded(params.collection_name, params.target, params.seed)?;

        let field_name = self
            .collection_field_name
            .as_deref()
            .unwrap_or("collection"); // Default collection field name is 'collection'.

        // TODO: Warn user if the collection field name would overwrite an existing field in a collection.

        for line_val in output.into_json_lines(field_name) {
            println!("{}", line_val);
        }

        Ok(())
    }
}
