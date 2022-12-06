use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::Namespace;

pub struct Scenario {
    namespace: Namespace,
    scenario: Namespace,
    name: String,
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

        Ok(self.namespace)
    }

    fn has_extra_collections(&self) -> Result<()> {
        let collections: Vec<_> = self.namespace.keys().collect();

        let extra_collections: Vec<_> = self
            .scenario
            .keys()
            .filter(|c| !collections.contains(c))
            .collect();

        if !extra_collections.is_empty() {
            let extra_collections = extra_collections
                .into_iter()
                .map(|e| format!("- {e}"))
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
        let scenario_collections: Vec<_> = self.scenario.keys().collect();

        let trim_collections: Vec<_> = self
            .namespace
            .keys()
            .map(ToOwned::to_owned)
            .into_iter()
            .filter(|c| !scenario_collections.contains(&c.as_str()))
            .collect();

        for trim_collection in trim_collections {
            debug!("removing collection '{}'", trim_collection);
            self.namespace.remove_collection(&trim_collection);
        }
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

    macro_rules! scenario {
        {
            $($inner:tt)*
        } => {
            serde_json::from_value::<crate::Namespace>(serde_json::json!($($inner)*))
                .expect("could not deserialize scenario into a namespace")
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
            namespace: scenario!({"collection1": {}, "collection2": {}}),
            scenario: scenario!({"collection1": {}}),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = scenario!({"collection1": {}});

        assert_eq!(actual, expected);
    }
}
