use anyhow::{Context, Result};
use dialoguer::Confirm;
use lazy_static::lazy_static;
use std::fs::DirEntry;
use std::path::PathBuf;
use std::str::FromStr;
use synth_core::schema::{Content, Name, Namespace};

lazy_static! {
    static ref UNDERLYING: Underlying = Underlying {
        file_ext: ".json".to_string(),
    };
}

struct Underlying {
    file_ext: String,
}

impl Underlying {
    fn parse(&self, text: &str) -> Result<Content> {
        serde_json::from_str(text).context("Failed to parse collection")
    }
}

pub struct Store {
    path: PathBuf,
}

impl Store {
    pub fn init() -> Result<Self> {
        Ok(Self {
            path: std::env::current_dir()
                .context(failed!(target: Debug, "Failed to initialise the store"))?,
        })
    }

    /// Visible for testing
    #[allow(unused)]
    fn with_dir(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn ns_exists(&self, path: &PathBuf) -> bool {
        path.exists()
    }

    /// Get a namespace given it's directory path
    pub fn get_ns(&self, ns_path: PathBuf) -> Result<Namespace> {
        let mut ns = Namespace::default();

        for entry in ns_path.read_dir()? {
            let entry = entry?;
            let (collection_name, content) = self
                .get_collection(&entry)
                .context(anyhow!("at file {}", entry.path().display()))?;
            ns.put_collection(&collection_name, content)?;
        }

        Ok(ns)
    }

    /// Save a namespace given it's directory path
    pub fn save_ns_path(&self, ns_path: PathBuf, namespace: Namespace) -> Result<()> {
        let mut collections = vec![];
        for (collection_name, collection) in namespace.collections.iter() {
            let collection_file_name =
                format!("{}{}", collection_name.to_string(), UNDERLYING.file_ext);
            let collection_path = ns_path.clone().join(collection_file_name);
            collections.push((collection_path, serde_json::to_string_pretty(collection)?))
        }

        if ns_path.exists() {
            // If the directory exists, warn the user that we are about to delete files in their FS
            // TODO for some reason this is not playing nice with the `wait_for_newline` option
            // Even without newline option the prompt is getting printed twice
            if !Confirm::new()
                //.wait_for_newline(true)
                .with_prompt(format!("To create the namespace, the directory `{}` and all of it's contents will be permanently deleted. Are you sure you want to perform this operation?", ns_path.display()))
                .interact()
                .unwrap() // unwrap here because the user can ctrl-c or something like that
            {
                // TODO Maybe an error here? Not sure tbh...
                std::process::exit(1);
            }
        }

        let _ = std::fs::remove_dir_all(ns_path.clone()).map_err(|_| trace!("Nothing to delete."));
        let _ = std::fs::create_dir_all(ns_path.clone())?;
        for (collection_path, collection) in collections {
            std::fs::write(collection_path, collection)?;
        }
        Ok(())
    }

    /// Visible for testing
    /// Save a namespace given it's proper name.
    /// So will save to <store-dir>/<name>
    #[allow(unused)]
    pub fn save_ns(&self, name: Name, namespace: Namespace) -> Result<()> {
        let ns_path = self.path.join(name.to_string());
        self.save_ns_path(ns_path, namespace)
    }

    fn get_collection(&self, dir_entry: &DirEntry) -> Result<(Name, Content)> {
        let entry_name = dir_entry.file_name();
        let file_name = entry_name.to_str().unwrap();
        if file_name.ends_with(&UNDERLYING.file_ext) {
            let collection_name = file_name.split(".").next().ok_or(failed!(
                target: Debug,
                "invalid filename {}",
                file_name
            ))?;
            let collection_file_content = std::fs::read_to_string(dir_entry.path())?;
            let collection = UNDERLYING.parse(&collection_file_content)?;
            return Ok((Name::from_str(collection_name)?, collection));
        } else {
            Err(failed!(target: Debug, "file is not of the right type"))
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_rw() -> Result<()> {
        let path: PathBuf = tempdir().unwrap().path().into();
        let store = Store::with_dir(path.clone());
        let ns = Namespace::default();
        let name = Name::from_str("users").unwrap();
        store.save_ns(name.clone(), ns.clone())?;

        let saved_ns = store.get_ns(path.join("users"))?;
        assert_eq!(saved_ns, ns);
        Ok(())
    }
}
