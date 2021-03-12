use anyhow::{Context, Result};
use lazy_static::lazy_static;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use synth_core::schema::{Content, Name, Namespace};

lazy_static! {
    static ref UNDERLYING: Underlying = Underlying {
        file_ext: "json".to_string(),
    };
}

struct Underlying {
    file_ext: String,
}

impl Underlying {
    fn extension(&self) -> &str {
        &self.file_ext
    }
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

    fn ns_path(&self, namespace: &Path) -> PathBuf {
        self.path.join(namespace)
    }

    pub fn relative_collection_path(namespace: &Path, collection: &Name) -> PathBuf {
        namespace
            .join(collection.to_string())
            .with_extension(UNDERLYING.extension())
    }

    fn collection_path(&self, namespace: &Path, collection: &Name) -> PathBuf {
        self.path
            .join(Self::relative_collection_path(namespace, collection))
    }

    pub fn ns_exists(&self, namespace: &Path) -> bool {
        self.ns_path(namespace).exists()
    }

    pub fn collection_exists(&self, namespace: &Path, collection: &Name) -> bool {
        self.collection_path(namespace, collection).exists()
    }

    /// Get a namespace given it's directory path
    pub fn get_ns(&self, ns_path: PathBuf) -> Result<Namespace> {
        let mut ns = Namespace::default();

        for entry in ns_path
            .read_dir()
            .context(format!("At path {:?}", ns_path))?
        {
            let entry = entry?;
            if let Some(file_ext) = entry.path().extension() {
                if file_ext == UNDERLYING.extension() {
                    let (collection_name, content) = self
                        .get_collection(&entry)
                        .context(anyhow!("at file {}", entry.path().display()))?;
                    ns.put_collection(&collection_name, content)?;
                }
            }
        }

        Ok(ns)
    }

    pub fn save_collection_path(
        &self,
        ns_path: &Path,
        collection: Name,
        content: Content,
    ) -> Result<()> {
        let abs_ns_path = self.ns_path(ns_path);
        std::fs::create_dir_all(&abs_ns_path)?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(self.collection_path(ns_path, &collection))?;
        serde_json::to_writer_pretty(&mut file, &content)?;
        Ok(())
    }

    /// Save a namespace given it's directory path
    pub fn save_ns_path(&self, ns_path: PathBuf, namespace: Namespace) -> Result<()> {
        let abs_ns_path = self.ns_path(&ns_path);
        std::fs::create_dir_all(&abs_ns_path)?;
        for (name, content) in namespace {
            self.save_collection_path(&ns_path, name, content)?;
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
        let collection_name = file_name.split(".").next().ok_or(failed!(
            target: Debug,
            "invalid filename {}",
            file_name
        ))?;
        let collection_file_content = std::fs::read_to_string(dir_entry.path())?;
        let collection = UNDERLYING
            .parse(&collection_file_content)
            .context(format!("At file {:?}", dir_entry.path()))?;
        return Ok((Name::from_str(collection_name)?, collection));
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
