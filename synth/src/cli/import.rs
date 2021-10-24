use std::collections::HashMap;
use std::convert::TryFrom;
use std::path::PathBuf;
use std::str::FromStr;

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
            .map(
                |format_string| match format_string.to_lowercase().as_str() {
                    "jsonl" => DataFormat::JsonLines {
                        collection_field_name,
                    },
                    "csv" => DataFormat::Csv,
                    _ => DataFormat::Json,
                },
            )
            .unwrap_or_default()
    }

    pub fn get_collection_field_name_or_default(&self) -> &str {
        match self {
            DataFormat::JsonLines {
                collection_field_name: Some(ref x),
            } => x,
            _ => "collection", // Default collection field name is 'collection'.
        }
    }
}

impl Default for DataFormat {
    fn default() -> Self {
        DataFormat::Json
    }
}

pub trait ImportStrategy {
    /// Import an entire namespace. Default implementation handles the importing of text-based formats (e.g. JSON, JSON
    /// Lines, CSV, provided `get_data_format`, `as_json_value`, `as_json_line_values` are implemented) - for database
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

            DataFormat::JsonLines { .. } => {
                let mut collection_names_to_values: HashMap<Option<String>, Vec<Value>> =
                    HashMap::new();

                for mut value in self.as_json_line_values()? {
                    match value {
                        Value::Object(ref mut obj_content) => {
                            let entry = {
                                if let Some(Value::String(collection_name)) = obj_content
                                    .remove(format.get_collection_field_name_or_default())
                                {
                                    collection_names_to_values
                                        .entry(Some(collection_name.to_string()))
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
                        let name_or_default = name.unwrap_or_else(|| "collection".to_string()); // TODO: Use --collection to give name

                        collection_from_values_jsonl(values)
                            .and_then(|content| Ok((name_or_default.parse()?, content)))
                            .with_context(|| {
                                anyhow!("While importing the collection '{}'", name_or_default)
                            })
                    })
                    .collect()
            }

            DataFormat::Csv => unimplemented!(),
        }
    }

    /// Get the format of text data being imported (JSON, JSON Lines, CSV) - used by the default implementation of
    /// `import` and not used by database integrations.
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

impl TryFrom<DataSourceParams> for Box<dyn ImportStrategy> {
    type Error = anyhow::Error;

    fn try_from(params: DataSourceParams) -> Result<Self, Self::Error> {
        match params.uri {
            None => Ok(Box::new(StdinImportStrategy {
                data_format: params.data_format,
            })),
            Some(uri) => {
                let import_strategy: Box<dyn ImportStrategy> = if uri.starts_with("postgres://")
                    || uri.starts_with("postgresql://")
                {
                    Box::new(PostgresImportStrategy {
                        uri,
                        schema: params.schema,
                    })
                } else if uri.starts_with("mongodb://") {
                    Box::new(MongoImportStrategy { uri })
                } else if uri.starts_with("mysql://") || uri.starts_with("mariadb://") {
                    Box::new(MySqlImportStrategy { uri })
                } else if let Ok(path) = PathBuf::from_str(&uri) {
                    Box::new(FileImportStrategy {
                        from_file: path,
                        data_format: params.data_format,
                    })
                } else {
                    return Err(anyhow!(
                         "Data source not recognized. Was expecting 'mongodb', 'postgres', 'mysql', or a file system path."
                    ));
                };
                Ok(import_strategy)
            }
        }
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
