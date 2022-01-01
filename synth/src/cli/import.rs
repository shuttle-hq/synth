use std::convert::TryFrom;
use std::path::PathBuf;

use anyhow::Result;

use synth_core::{Content, DataSourceParams};

use crate::cli::csv::{CsvFileImportStrategy, CsvStdinImportStrategy};
use crate::cli::json::{JsonFileImportStrategy, JsonStdinImportStrategy};
use crate::cli::jsonl::{JsonLinesFileImportStrategy, JsonLinesStdinImportStrategy};
use crate::cli::mongo::MongoImportStrategy;
use crate::cli::mysql::MySqlImportStrategy;
use crate::cli::postgres::PostgresImportStrategy;

use super::map_from_uri_query;

pub trait ImportStrategy {
    /// Import an entire namespace.
    fn import_namespace(&self) -> Result<Content>;

    /// Import a single collection. Default implementation works by calling `import` and then extracting from the
    /// returned namespace the correct collection based on the `name` parameter.
    fn import_collection(&self, name: &str) -> Result<Content> {
        self.import_namespace()?.remove_collection(name)
    }
}

impl TryFrom<DataSourceParams<'_>> for Box<dyn ImportStrategy> {
    type Error = anyhow::Error;

    fn try_from(params: DataSourceParams) -> Result<Self, Self::Error> {
        let scheme = params.uri.scheme().as_str().to_lowercase();
        let query = map_from_uri_query(params.uri.query());

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
                let collection_field_name = query
                    .get("collection_field_name")
                    .unwrap_or(&"type")
                    .to_string();

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
            "csv" => {
                // TODO: Would rather have this work as a flag e.g. `csv:directory?no_header_row` implies no header row
                // in CSV data, otherwise assume there will be one.
                let expect_header_row = query
                    .get("header_row")
                    .map(|x| *x != "false")
                    .unwrap_or(true);

                if params.uri.path() == "" {
                    Box::new(CsvStdinImportStrategy { expect_header_row })
                } else {
                    Box::new(CsvFileImportStrategy {
                        from_dir: PathBuf::from(params.uri.path().to_string()),
                        expect_header_row,
                    })
                }
            }
            _ => {
                return Err(anyhow!(
                    "Import URI scheme not recognised. Was expecting one of 'mongodb', 'postgres', 'mysql', 'mariadb', 'json', 'jsonl', or 'csv'."
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
    use crate::cli::csv::import_csv_collection;

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

    fn json_csv_equiv_assert(csv: &str, json: serde_json::Value) {
        let from_csv =
            import_csv_collection(csv::Reader::from_reader(csv.as_bytes()), true).unwrap();
        let from_json = import_json(json)
            .unwrap()
            .get_collection("collection")
            .unwrap()
            .clone();

        assert_eq!(from_csv, from_json);
    }

    #[test]
    fn test_json_and_csv_import_equivalence() {
        json_csv_equiv_assert(
            concat!(
                "a,b[0].c,b[0].d,b[1].c,b[1].d,e.f[0],e.f[1],e.f[2],e.g\n",
                "10,true,3.56,,,1,2,3,\n",
                "25,false,-12.5,true,45.3,1,,,5"
            ),
            serde_json::json!({
                "collection": [
                    {
                        "a": 10,
                        "b": [
                            { "c": true, "d": 3.56 },
                            { "c": null, "d": null }
                        ],
                        "e": {
                            "f": [1, 2, 3],
                            "g": null
                        }
                    },
                    {
                        "a": 25,
                        "b": [
                            { "c": false, "d": -12.5 },
                            { "c": true, "d": 45.3 }
                        ],
                        "e": {
                            "f": [1, null, null],
                            "g": 5
                        }
                    }
                ]
            }),
        );

        json_csv_equiv_assert(
            concat!(
                "[0].x,[0].y[0],[0].y[1],[1].x,[1].y[0],[1].y[1]\n",
                "10.2,abc,def,-12.5,ghi,jkl\n",
                "m,foo,bar,n,bar,foo"
            ),
            serde_json::json!({
                "collection": [
                    [ { "x": 10.2, "y": ["abc", "def"] }, { "x": -12.5, "y": ["ghi", "jkl"] } ],
                    [ { "x": "m", "y": ["foo", "bar"] }, { "x": "n", "y": ["bar", "foo"] } ]
                ]
            }),
        );
    }
}
