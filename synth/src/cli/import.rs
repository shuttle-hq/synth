use std::collections::HashMap;
use std::convert::TryFrom;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_json::Value;

use synth_core::graph::prelude::{MergeStrategy, OptionalMergeStrategy};
use synth_core::schema::Namespace;
use synth_core::{Content, Name};

use crate::cli::db_utils::DataSourceParams;
use crate::cli::mongo::MongoImportStrategy;
use crate::cli::mysql::MySqlImportStrategy;
use crate::cli::postgres::PostgresImportStrategy;
use crate::cli::stdf::{FileImportStrategy, StdinImportStrategy};

use super::DataFormat;

pub trait ImportStrategy {
    /// Import an entire namespace. Default implementation handles the importing of text-based formats (e.g. JSON, JSON
    /// Lines, provided `get_data_format`, `as_json_value`, `as_json_line_values` are implemented) - for database
    /// integrations this function should be overridden.
    fn import(&self) -> Result<Namespace> {
        let format = self.get_data_format();

        match format {
            DataFormat::Json => match self.as_json_value()? {
                Value::Object(object) => object
                    .into_iter()
                    .map(|(name, value)| {
                        collection_from_value(&value)
                            .and_then(|content| Ok((name.parse()?, content)))
                            .with_context(|| anyhow!("While importing the collection `{}`", name))
                    })
                    .collect(),
                unacceptable => Err(anyhow!(
                    "Was expecting an object, instead got `{}`",
                    unacceptable
                )),
            },

            DataFormat::JsonLines {
                collection_field_name,
            } => {
                let mut collection_names_to_values: HashMap<Option<String>, Vec<Value>> =
                    HashMap::new();

                for mut value in self.as_json_line_values()? {
                    match value {
                        Value::Object(ref mut obj_content) => {
                            let entry = {
                                if let Some(Value::String(collection_name)) =
                                    obj_content.remove(collection_field_name)
                                {
                                    collection_names_to_values.entry(Some(collection_name))
                                } else {
                                    collection_names_to_values.entry(None)
                                }
                            }
                            .or_default();

                            entry.push(value);
                        }
                        _ => {
                            collection_names_to_values
                                .entry(None)
                                .or_default()
                                .push(value);
                        }
                    }
                }

                collection_names_to_values
                    .into_iter()
                    .map(|(name, values)| {
                        let name_or_default = name.unwrap_or_else(|| "collection".to_string());

                        collection_from_values_jsonl(values)
                            .and_then(|content| Ok((name_or_default.parse()?, content)))
                            .with_context(|| {
                                anyhow!("While importing the collection '{}'", name_or_default)
                            })
                    })
                    .collect()
            }
        }
    }

    /// Get the format of text data being imported (JSON, JSON Lines) - used by the default implementation of `import`
    /// and not used by database integrations.
    fn get_data_format(&self) -> &DataFormat {
        unreachable!()
    }

    /// Get the JSON data to be imported - called by the default implementation of `import` when dealing with JSON data.
    /// Not used by database integrations.
    fn as_json_value(&self) -> Result<Value> {
        unreachable!()
    }

    /// Get the JSON Lines data to be imported (as a vector of JSON values) - called by the default implementation of
    /// `import` when dealing with JSON Lines data. Not used by database integrations.
    fn as_json_line_values(&self) -> Result<Vec<Value>> {
        unreachable!()
    }

    /// Import a single collection.
    fn import_collection(&self, name: &Name) -> Result<Content> {
        self.import()?
            .collections
            .remove(name)
            .ok_or_else(|| anyhow!("Could not find collection '{}'.", name))
    }
}

impl TryFrom<DataSourceParams<'_>> for Box<dyn ImportStrategy> {
    type Error = anyhow::Error;

    fn try_from(params: DataSourceParams) -> Result<Self, Self::Error> {
        let scheme = params.uri.scheme().as_str().to_lowercase();
        let import_strategy: Box<dyn ImportStrategy> = match scheme.as_str() {
            "postgres" | "postgresql" => Box::new(PostgresImportStrategy {
                uri_string: params.uri.to_string(),
                schema: params.schema,
            }),
            "mongodb" => Box::new(MongoImportStrategy {
                uri_string: params.uri.to_string(),
            }),
            "mysql" | "mariadb" => Box::new(MySqlImportStrategy {
                uri_string: params.uri.to_string(),
            }),
            "json" | "jsonl" => {
                let data_format = DataFormat::new(
                    &scheme,
                    params
                        .uri
                        .query()
                        .map(uriparse::Query::as_str)
                        .unwrap_or_default(),
                );

                if params.uri.path() == "" {
                    Box::new(StdinImportStrategy { data_format })
                } else {
                    Box::new(FileImportStrategy {
                        data_format,
                        from_file: PathBuf::from(params.uri.path().to_string()),
                    })
                }
            }
            _ => {
                return Err(anyhow!(
                    "Import URI scheme not recognised. Was expecting one of 'mongodb', 'postgres', 'mysql', 'mariadb', 'json' or 'jsonl'."
                ));
            }
        };
        Ok(import_strategy)
    }
}

fn collection_from_value(value: &Value) -> Result<Content> {
    match value {
        Value::Array(values) => {
            let fst = values.first().unwrap_or(&Value::Null);
            let mut as_content = Namespace::collection(fst);
            OptionalMergeStrategy.try_merge(&mut as_content, value)?;
            Ok(as_content)
        }
        unacceptable => Err(anyhow!(
            "Was expecting a collection, instead got `{}`",
            unacceptable
        )),
    }
}

/// Create a collection (`Content`) from a set of Serde JSON values that were all generated originally from the same
/// collection.
fn collection_from_values_jsonl(values: Vec<Value>) -> Result<Content> {
    let fst = values.first().unwrap_or(&Value::Null);
    let mut as_content = Namespace::collection(fst);
    OptionalMergeStrategy.try_merge(&mut as_content, &Value::Array(values))?;
    Ok(as_content)
}
