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
                let trim_fields: Vec<_> = map_keys
                    .into_iter()
                    .filter(|c| !fields.contains_key(c.as_str()))
                    .map(ToOwned::to_owned)
                    .collect();

                for trim_field in trim_fields {
                    debug!("removing field '{}'", trim_field);
                    map.fields.remove(trim_field.as_str());
                }
            }
            Content::Array(arr) => {
                Self::trim_collection_fields(&mut arr.content, fields)?;
            }
            _ => return Err(anyhow!("cannot select fields to include from a non-object")),
        };

        Ok(())
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
}
