use anyhow::{Context, Result};
use serde::Deserialize;
use std::{collections::BTreeMap, path::PathBuf};

use crate::{Content, Namespace};

pub struct Scenario {
    namespace: Namespace,
    scenario: ScenarioNamespace,
    name: String,
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
struct ScenarioNamespace {
    #[serde(flatten)]
    collections: BTreeMap<String, ScenarioCollection>,
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
struct ScenarioCollection {
    #[serde(flatten)]
    fields: BTreeMap<String, Content>,
}

impl Scenario {
    pub fn new(namespace: Namespace, namespace_path: PathBuf, scenario: &str) -> Result<Self> {
        let scenario_path = namespace_path
            .join("scenarios")
            .join(scenario)
            .with_extension("json");

        if !scenario_path.exists() {
            return Err(anyhow!(
                "could not find scenario at {}",
                scenario_path.display()
            ));
        }

        let scenario_content = std::fs::read_to_string(scenario_path.clone())?;
        debug!("found scenario:\n{}", scenario);
        let scenario_namespace = serde_json::from_str(&scenario_content).context(anyhow!(
            "Failed to parse scenario '{}'",
            scenario_path.display()
        ))?;

        Ok(Self {
            namespace,
            scenario: scenario_namespace,
            name: scenario.to_string(),
        })
    }

    pub fn build(mut self) -> Result<Namespace> {
        self.has_extra_collections()
            .context(anyhow!("failed to build scenario '{}'", self.name))?;
        self.trim_namespace_collections();
        self.trim_fields()?;

        Ok(self.namespace)
    }

    fn has_extra_collections(&self) -> Result<()> {
        let collections: Vec<_> = self.namespace.keys().collect();

        let extra_collections: Vec<_> = self
            .scenario
            .collections
            .keys()
            .filter(|c| !collections.contains(&c.as_str()))
            .collect();

        if !extra_collections.is_empty() {
            let extra_collections = extra_collections
                .into_iter()
                .map(|e| format!("- {}", e))
                .collect::<Vec<String>>()
                .join("\n");

            return Err(anyhow!(
                "the namespace does not contain the following collection(s):\n{}",
                extra_collections
            ));
        }

        Ok(())
    }

    fn trim_namespace_collections(&mut self) {
        let scenario_collections: Vec<_> = self.scenario.collections.keys().collect();

        let trim_collections: Vec<_> = self
            .namespace
            .keys()
            .map(ToOwned::to_owned)
            .into_iter()
            .filter(|c| !scenario_collections.contains(&c))
            .collect();

        for trim_collection in trim_collections {
            debug!("removing collection '{}'", trim_collection);
            self.namespace.remove_collection(&trim_collection);
        }
    }

    fn trim_fields(&mut self) -> Result<()> {
        for (name, collection) in self.scenario.collections.iter() {
            // Nothing to trim
            if collection.fields.is_empty() {
                continue;
            }

            let namespace_collection = self.namespace.get_collection_mut(name)?;

            Self::trim_collection_fields(namespace_collection, &collection.fields)
                .context(anyhow!("failed to trim collection '{}'", name))?;
        }

        Ok(())
    }

    fn trim_collection_fields(
        collection: &mut Content,
        fields: &BTreeMap<String, Content>,
    ) -> Result<()> {
        match collection {
            Content::Object(map) => {
                let map_keys: Vec<_> = map.fields.keys().collect();

                for field in fields.keys() {
                    if !map_keys.contains(&field) {
                        return Err(anyhow!(
                            "'{}' is not a field on the object, therefore it cannot be included",
                            field
                        ));
                    }
                }
                let (keep_fields, trim_fields): (Vec<_>, Vec<_>) = map_keys
                    .into_iter()
                    .partition(|c| fields.contains_key(c.as_str()));

                let trim_fields: Vec<_> = trim_fields.into_iter().map(ToOwned::to_owned).collect();
                let keep_fields: Vec<_> = keep_fields.into_iter().map(ToOwned::to_owned).collect();

                for trim_field in trim_fields {
                    debug!("removing field '{}'", trim_field);
                    map.fields.remove(trim_field.as_str());
                }

                for keep_field in keep_fields {
                    // Safe to unwrap since we already checked the existence of this field in the
                    // partition step
                    let overwrite = fields.get(&keep_field).unwrap();

                    // If this just an include
                    if let Content::Empty(_) = overwrite {
                        continue;
                    }
                    debug!("merging field '{}'", keep_field);

                    // Safe to unwrap since we got `keep_field` from `map.fields`
                    let original = map.fields.get_mut(&keep_field).unwrap();
                    Self::merge_field(original, overwrite)
                        .context(anyhow!("failed to overwrite field '{}'", keep_field))?;
                }
            }
            Content::Array(arr) => {
                Self::trim_collection_fields(&mut arr.content, fields)?;
            }
            _ => return Err(anyhow!("cannot select fields to include from a non-object")),
        };

        Ok(())
    }

    fn merge_field(original: &mut Content, overwrite: &Content) -> Result<()> {
        // We check if types are the same first to find redundant overwrites. Else it must be a type overwrite.
        match (&original, overwrite) {
            (Content::Null(orig), Content::Null(over)) if orig == over => return Self::same_err(),
            (Content::Bool(orig), Content::Bool(over)) if orig == over => return Self::same_err(),
            (Content::Number(orig), Content::Number(over)) if orig == over => {
                return Self::same_err()
            }
            (Content::String(orig), Content::String(over)) if orig == over => {
                return Self::same_err()
            }
            (Content::DateTime(orig), Content::DateTime(over)) if orig == over => {
                return Self::same_err()
            }
            (Content::OneOf(orig), Content::OneOf(over)) if orig == over => {
                return Self::same_err()
            }
            _ => *original = overwrite.clone(),
        }

        Ok(())
    }

    fn same_err() -> Result<()> {
        Err(anyhow!("overwrite is same as original"))
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::{
        fs::{self, File},
        io::Write,
        path::PathBuf,
    };

    use tempfile::tempdir;

    use super::Scenario;

    macro_rules! namespace {
        {
            $($inner:tt)*
        } => {
            serde_json::from_value::<crate::Namespace>(serde_json::json!($($inner)*))
                .expect("could not deserialize into a namespace")
        }
    }

    macro_rules! scenario {
        {
            $($inner:tt)*
        } => {
            serde_json::from_value::<super::ScenarioNamespace>(serde_json::json!($($inner)*))
                .expect("could not deserialize into a scenario namespace")
        }
    }

    #[test]
    #[should_panic(expected = "could not find scenario")]
    fn new_missing() {
        let path = tempdir().unwrap().path().into();
        Scenario::new(Default::default(), path, "missing").unwrap();
    }

    #[test]
    fn new_exist() -> Result<()> {
        let path: PathBuf = tempdir()?.path().into();

        let scenario_dir = path.join("scenarios");
        fs::create_dir_all(&scenario_dir)?;

        let scenario_path = scenario_dir.join("dummy").with_extension("json");
        let mut file = File::create(scenario_path)?;
        write!(file, r#"{{ "collection": {{}} }}"#)?;

        let scenario = Scenario::new(Default::default(), path, "dummy").unwrap();

        let expected = scenario!({"collection": {}});

        assert_eq!(scenario.scenario, expected);

        Ok(())
    }

    #[test]
    #[should_panic(
        expected = "the namespace does not contain the following collection(s):\n    - extra"
    )]
    fn build_extra_collection() {
        let scenario = Scenario {
            namespace: Default::default(),
            scenario: scenario!({"extra": {}}),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    fn build_filter_collections() {
        let scenario = Scenario {
            namespace: namespace!({"collection1": {}, "collection2": {}}),
            scenario: scenario!({"collection1": {}}),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({"collection1": {}});

        assert_eq!(actual, expected);
    }

    #[test]
    fn build_filter_fields() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection1": {
                    "type": "object",
                    "nully": {"type": "null"},
                    "stringy": {"type": "string", "pattern": "test"}
                },
                "collection2": {}
            }),
            scenario: scenario!({"collection1": {"nully": {}}}),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection1": {
                "type": "object",
                "nully": {"type": "null"}
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn build_filter_fields_array() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection1": {
                    "type": "array",
                    "length": 5,
                    "content": {
                        "type": "object",
                        "nully": {"type": "null"},
                        "stringy": {"type": "string", "pattern": "test"}
                    }
                },
                "collection2": {}
            }),
            scenario: scenario!({"collection1": {"nully": {}}}),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection1": {
                "type": "array",
                "length": 5,
                "content": {
                    "type": "object",
                    "nully": {"type": "null"},
                }
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "'null' is not a field on the object")]
    fn build_filter_extra_field() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection1": {
                    "type": "object",
                    "nully": {"type": "null"},
                    "stringy": {"type": "string", "pattern": "test"}
                },
                "collection2": {}
            }),
            scenario: scenario!({"collection1": {"null": {}}}),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    #[should_panic(expected = "cannot select fields to include from a non-object")]
    fn build_filter_field_scalar() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection1": {
                    "type": "null"
                },
            }),
            scenario: scenario!({"collection1": {"nully": {}}}),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    fn build_overwrite_types() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "nully": {"type": "null"},
                    "stringy": {"type": "string", "pattern": "test"}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "nully": {"type": "string", "pattern": "test"},
                    "stringy": {"type": "null"}
                }
            }),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection": {
                "type": "object",
                "nully": {"type": "string", "pattern": "test"},
                "stringy": {"type": "null"}
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "overwrite is same as original")]
    fn build_overwrite_null() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "nully": {"type": "null"}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "nully": {"type": "null"}
                }
            }),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    fn build_overwrite_bool() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "bool_constant": {"type": "bool", "constant": true},
                    "bool_frequency": {"type": "bool", "frequency": 0.5},
                    "bool_subtype": {"type": "bool", "constant": true}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "bool_constant": {"type": "bool", "constant": false},
                    "bool_frequency": {"type": "bool", "frequency": 0.3},
                    "bool_subtype": {"type": "bool", "frequency": 0.8}
                }
            }),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection": {
                "type": "object",
                "bool_constant": {"type": "bool", "constant": false},
                "bool_frequency": {"type": "bool", "frequency": 0.3},
                "bool_subtype": {"type": "bool", "frequency": 0.8}
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "overwrite is same as original")]
    fn build_overwrite_bool_same() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "same": {"type": "bool", "constant": true}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "same": {"type": "bool", "constant": true}
                }
            }),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    fn build_overwrite_number() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "number_u32_constant": {"type": "number", "subtype": "u32", "constant": 2},
                    "number_u32_range": {"type": "number", "subtype": "u32", "range": {"low": 0, "high": 5, "step": 1}},
                    "number_u32_id": {"type": "number", "subtype": "u32", "id": {"start_at": 3}},
                    "number_u64_constant": {"type": "number", "subtype": "u64", "constant": 2},
                    "number_u64_range": {"type": "number", "subtype": "u64", "range": {"low": 0, "high": 5, "step": 1}},
                    "number_u64_id": {"type": "number", "subtype": "u64", "id": {"start_at": 3}},
                    "number_i32_constant": {"type": "number", "subtype": "i32", "constant": -2},
                    "number_i32_range": {"type": "number", "subtype": "i32", "range": {"low": -3, "high": 5, "step": 1}},
                    "number_i32_id": {"type": "number", "subtype": "i32", "id": {"start_at": -3}},
                    "number_i64_constant": {"type": "number", "subtype": "i64", "constant": -2},
                    "number_i64_range": {"type": "number", "subtype": "i64", "range": {"low": -3, "high": 5, "step": 1}},
                    "number_i64_id": {"type": "number", "subtype": "i64", "id": {"start_at": -3}},
                    "number_f32_constant": {"type": "number", "subtype": "f32", "constant": 3.2},
                    "number_f32_range": {"type": "number", "subtype": "f32", "range": {"low": 3.1, "high": 5.3, "step": 0.1}},
                    "number_f64_constant": {"type": "number", "subtype": "f64", "constant": 4.2},
                    "number_f64_range": {"type": "number", "subtype": "f64", "range": {"low": 34.2, "high": 56.3, "step": 0.3}}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "number_u32_constant": {"type": "number", "subtype": "u32", "constant": 3},
                    "number_u32_range": {"type": "number", "subtype": "u32", "range": {"low": 2, "high": 10, "step": 3}},
                    "number_u32_id": {"type": "number", "subtype": "u32", "id": {"start_at": 7}},
                    "number_u64_constant": {"type": "number", "subtype": "u64", "constant": 8},
                    "number_u64_range": {"type": "number", "subtype": "u64", "range": {"low": 4, "high": 10, "step": 2}},
                    "number_u64_id": {"type": "number", "subtype": "u64", "id": {"start_at": 1}},
                    "number_i32_constant": {"type": "number", "subtype": "i32", "constant": -7},
                    "number_i32_range": {"type": "number", "subtype": "i32", "range": {"low": -8, "high": 7, "step": 2}},
                    "number_i32_id": {"type": "number", "subtype": "i32", "id": {"start_at": -2}},
                    "number_i64_constant": {"type": "number", "subtype": "i64", "constant": -9},
                    "number_i64_range": {"type": "number", "subtype": "i64", "range": {"low": -8, "high": 7, "step": 2}},
                    "number_i64_id": {"type": "number", "subtype": "i64", "id": {"start_at": -6}},
                    "number_f32_constant": {"type": "number", "subtype": "f32", "constant": 6.2},
                    "number_f32_range": {"type": "number", "subtype": "f32", "range": {"low": 4.2, "high": 8.3, "step": 0.7}},
                    "number_f64_constant": {"type": "number", "subtype": "f64", "constant": 6.3},
                    "number_f64_range": {"type": "number", "subtype": "f64", "range": {"low": 3.2, "high": 5.3, "step": 0.04}}
                }
            }),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection": {
                "type": "object",
                "number_u32_constant": {"type": "number", "subtype": "u32", "constant": 3},
                "number_u32_range": {"type": "number", "subtype": "u32", "range": {"low": 2, "high": 10, "step": 3}},
                "number_u32_id": {"type": "number", "subtype": "u32", "id": {"start_at": 7}},
                "number_u64_constant": {"type": "number", "subtype": "u64", "constant": 8},
                "number_u64_range": {"type": "number", "subtype": "u64", "range": {"low": 4, "high": 10, "step": 2}},
                "number_u64_id": {"type": "number", "subtype": "u64", "id": {"start_at": 1}},
                "number_i32_constant": {"type": "number", "subtype": "i32", "constant": -7},
                "number_i32_range": {"type": "number", "subtype": "i32", "range": {"low": -8, "high": 7, "step": 2}},
                "number_i32_id": {"type": "number", "subtype": "i32", "id": {"start_at": -2}},
                "number_i64_constant": {"type": "number", "subtype": "i64", "constant": -9},
                "number_i64_range": {"type": "number", "subtype": "i64", "range": {"low": -8, "high": 7, "step": 2}},
                "number_i64_id": {"type": "number", "subtype": "i64", "id": {"start_at": -6}},
                "number_f32_constant": {"type": "number", "subtype": "f32", "constant": 6.2},
                "number_f32_range": {"type": "number", "subtype": "f32", "range": {"low": 4.2, "high": 8.3, "step": 0.7}},
                "number_f64_constant": {"type": "number", "subtype": "f64", "constant": 6.3},
                "number_f64_range": {"type": "number", "subtype": "f64", "range": {"low": 3.2, "high": 5.3, "step": 0.04}}
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "overwrite is same as original")]
    fn build_overwrite_number_same() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "same": {"type": "number", "constant": 2}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "same": {"type": "number", "constant": 2}
                }
            }),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    fn build_overwrite_string() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "string_pattern": {"type": "string", "pattern": "(m|f)"},
                    "string_format": {"type": "string", "format": {"format": "the lucky number is {number}", "arguments": {"number": 5}}},
                    "string_faker": {"type": "string", "faker": {"generator": "address"}},
                    "string_serialized": {"type": "string", "serialized": {"serializer": "json", "content": 3}},
                    "string_truncated": {"type": "string", "truncated": {"length": 4, "content": {"type": "string", "pattern": "[a-z]{30}"}}},
                    "string_sliced": {"type": "string", "sliced": {"slice": "4:8", "content": {"type": "string", "pattern": "[a-z]{30}"}}},
                    "string_constant": {"type": "string", "constant": "hello world"},
                    "string_categorical": {"type": "string", "categorical": {"hello": 4, "world": 3}}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "string_pattern": {"type": "string", "pattern": "(male|female)"},
                    "string_format": {"type": "string", "format": {"format": "{number} was guessed", "arguments": {"number": 2}}},
                    "string_faker": {"type": "string", "faker": {"generator": "first_name"}},
                    "string_serialized": {"type": "string", "serialized": {"serializer": "json", "content": 7}},
                    "string_truncated": {"type": "string", "truncated": {"length": 8, "content": {"type": "string", "pattern": "[A-Z]{35}"}}},
                    "string_sliced": {"type": "string", "sliced": {"slice": "25:30", "content": {"type": "string", "pattern": "[A-Z]{63}"}}},
                    "string_constant": {"type": "string", "constant": "bye world"},
                    "string_categorical": {"type": "string", "categorical": {"bye": 8, "world": 6}}
                }
            }),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection": {
                "type": "object",
                "string_pattern": {"type": "string", "pattern": "(male|female)"},
                "string_format": {"type": "string", "format": {"format": "{number} was guessed", "arguments": {"number": 2}}},
                "string_faker": {"type": "string", "faker": {"generator": "first_name"}},
                "string_serialized": {"type": "string", "serialized": {"serializer": "json", "content": 7}},
                "string_truncated": {"type": "string", "truncated": {"length": 8, "content": {"type": "string", "pattern": "[A-Z]{35}"}}},
                "string_sliced": {"type": "string", "sliced": {"slice": "25:30", "content": {"type": "string", "pattern": "[A-Z]{63}"}}},
                "string_constant": {"type": "string", "constant": "bye world"},
                "string_categorical": {"type": "string", "categorical": {"bye": 8, "world": 6}}
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "overwrite is same as original")]
    fn build_overwrite_string_same() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "string_uuid": {"type": "string", "uuid": {}}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "string_uuid": {"type": "string", "uuid": {}}
                }
            }),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    fn build_overwrite_date_time() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "date_time_naive_date": {"type": "date_time", "subtype": "naive_date", "format": "%Y-%m-%d", "begin": "2022-02-15", "end": "2022-02-19"},
                    "date_time_naive_time": {"type": "date_time", "subtype": "naive_time", "format": "%H:%M:%S", "begin": "13:34:34", "end": "14:32:53"},
                    "date_time_naive_date_time": {"type": "date_time", "subtype": "naive_date_time", "format": "%Y-%m-%dT%H:%M:%S", "begin": "2022-03-23T13:34:34", "end": "2022-04-30T3:32:53"},
                    "date_time_date_time": {"type": "date_time", "subtype": "date_time", "format": "%Y-%m-%dT%H:%M:%S%z", "begin": "2022-03-23T13:34:34+0100", "end": "2022-04-30T3:32:53+0100"}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "date_time_naive_date": {"type": "date_time", "subtype": "naive_date", "format": "%Y-%m-%d", "begin": "2022-01-31", "end": "2022-02-23"},
                    "date_time_naive_time": {"type": "date_time", "subtype": "naive_time", "format": "%H:%M:%S", "begin": "22:32:35", "end": "23:34:35"},
                    "date_time_naive_date_time": {"type": "date_time", "subtype": "naive_date_time", "format": "%Y-%m-%dT%H:%M:%S", "begin": "2022-02-25T11:49:39", "end": "2022-03-12T12:39:28"},
                    "date_time_date_time": {"type": "date_time", "subtype": "date_time", "format": "%Y-%m-%dT%H:%M:%S%z", "begin": "2022-02-25T11:49:39+0000", "end": "2022-03-12T12:39:28+0000"}
                }
            }),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection": {
                "type": "object",
                "date_time_naive_date": {"type": "date_time", "subtype": "naive_date", "format": "%Y-%m-%d", "begin": "2022-01-31", "end": "2022-02-23"},
                "date_time_naive_time": {"type": "date_time", "subtype": "naive_time", "format": "%H:%M:%S", "begin": "22:32:35", "end": "23:34:35"},
                "date_time_naive_date_time": {"type": "date_time", "subtype": "naive_date_time", "format": "%Y-%m-%dT%H:%M:%S", "begin": "2022-02-25T11:49:39", "end": "2022-03-12T12:39:28"},
                "date_time_date_time": {"type": "date_time", "subtype": "date_time", "format": "%Y-%m-%dT%H:%M:%S%z", "begin": "2022-02-25T11:49:39+0000", "end": "2022-03-12T12:39:28+0000"}
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "overwrite is same as original")]
    fn build_overwrite_date_time_same() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "same": {"type": "date_time", "subtype": "naive_date", "format": "%Y-%m-%d", "begin": "2022-02-15", "end": "2022-02-19"}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "same": {"type": "date_time", "subtype": "naive_date", "format": "%Y-%m-%d", "begin": "2022-02-15", "end": "2022-02-19"}
                }
            }),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    fn build_overwrite_one_of() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "one_of": {"type": "one_of", "variants": [{"weight": 0.5, "type": "string", "pattern": "m|f"}, {"weight": 0.5, "type": "null"}]}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "one_of": {"type": "one_of", "variants": [{"weight": 0.8, "type": "string", "pattern": "m|f"}, {"weight": 0.2, "type": "null"}]}
                }
            }),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection": {
                "type": "object",
                "one_of": {"type": "one_of", "variants": [{"weight": 0.8, "type": "string", "pattern": "m|f"}, {"weight": 0.2, "type": "null"}]}
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "overwrite is same as original")]
    fn build_overwrite_one_of_same() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "same": {"type": "one_of", "variants": [{"weight": 0.5, "type": "string", "pattern": "m|f"}, {"weight": 0.5, "type": "null"}]}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "same": {"type": "one_of", "variants": [{"weight": 0.5, "type": "null"}, {"weight": 0.5, "type": "string", "pattern": "m|f"}]}
                }
            }),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }
}
