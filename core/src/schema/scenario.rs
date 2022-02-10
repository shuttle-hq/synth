use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::Namespace;

pub struct Scenario {
    namespace: Namespace,
    scenario: Namespace,
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

        let scenario_content = std::fs::read_to_string(scenario_path)?;
        let scenario =
            serde_json::from_str(&scenario_content).context("Failed to parse collection")?;

        Ok(Self {
            namespace,
            scenario,
        })
    }

    pub fn build(self) -> Namespace {
        self.namespace
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
}
