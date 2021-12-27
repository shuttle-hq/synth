use std::convert::TryFrom;
use std::path::PathBuf;

use anyhow::Result;

use synth_core::Content;

use crate::cli::db_utils::DataSourceParams;
use crate::cli::json::{JsonFileImportStrategy, JsonStdinImportStrategy};
use crate::cli::jsonl::{JsonLinesFileImportStrategy, JsonLinesStdinImportStrategy};
use crate::cli::mongo::MongoImportStrategy;
use crate::cli::mysql::MySqlImportStrategy;
use crate::cli::postgres::PostgresImportStrategy;

use super::collection_field_name_from_uri_query;

pub trait ImportStrategy {
    /// Import an entire namespace.
    fn import(&self) -> Result<Namespace>;

    /// Import a single collection. Default implementation works by calling `import` and then extracting from the
    /// returned namespace the correct collection based on the `name` parameter.
    fn import_collection(&self, name: &str) -> Result<Content> {
        self.import()?
            .remove_collection(name)
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
            "json" => {
                if params.uri.path() == "" {
                    Box::new(JsonStdinImportStrategy)
                } else {
                    Box::new(JsonFileImportStrategy {
                        from_file: PathBuf::from(params.uri.path().to_string()),
                    })
                }
            }
            "jsonl" => {
                let collection_field_name =
                    collection_field_name_from_uri_query(params.uri.query());

                if params.uri.path() == "" {
                    Box::new(JsonLinesStdinImportStrategy {
                        collection_field_name,
                    })
                } else {
                    Box::new(JsonLinesFileImportStrategy {
                        from_file: PathBuf::from(params.uri.path().to_string()),
                        collection_field_name,
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

#[cfg(test)]
mod tests {
    use crate::cli::json::import_json;
    use crate::cli::jsonl::import_json_lines;

    #[test]
    fn test_json_and_json_lines_import_equivalence() {
        let json_lines = vec![
            serde_json::json!({"type": "first", "num": 10, "float": 0.025}),
            serde_json::json!({"type": "first", "num": 25, "float": 2.3}),
            serde_json::json!({"type": "second", "obj": {"first": "John", "second": "Doe"}}),
            serde_json::json!({"type": "first", "num": 16, "float": 25.0002, "optional": true}),
        ];

        let json = serde_json::json!({
            "first": [
                {"num": 10, "float": 0.025},
                {"num": 25, "float": 2.3},
                {"num": 16, "float": 25.0002, "optional": true}
            ],
            "second": [
                {"obj": {"first": "John", "second": "Doe"}}
            ]
        });

        assert_eq!(
            import_json_lines(json_lines, "type").unwrap(),
            import_json(json).unwrap()
        );
    }
}
