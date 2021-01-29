use anyhow::Result;
use serde_json::{Map, Value};
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use synth_core::schema::Namespace;
use synth_core::Name;

pub(crate) trait ImportStrategy {
    fn import(self) -> Result<Namespace>;
}

#[derive(Clone, Debug)]
pub(crate) enum SomeImportStrategy {
    StdinImportStrategy(StdinImportStrategy),
    FromFile(FileImportStrategy),
    #[allow(unused)]
    FromPostgres(PostgresImportStrategy),
}

impl Default for SomeImportStrategy {
    fn default() -> Self {
        SomeImportStrategy::StdinImportStrategy(StdinImportStrategy {})
    }
}

impl FromStr for SomeImportStrategy {
    type Err = anyhow::Error;

    /// Here we exhaustively try to pattern match strings until we get something
    /// that successfully parses. Starting from a file, could be a url to a database etc.
    /// We assume that these can be unambiguously identified for now.
    /// For example, `postgres://...` is not going to be a file on the FS
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(path) = PathBuf::from_str(s) {
            return Ok(SomeImportStrategy::FromFile(FileImportStrategy {
                from_file: path,
            }));
        }
        Err(anyhow!("Data source not recognized"))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PostgresImportStrategy {
    uri: String,
}

#[derive(Clone, Debug)]
pub(crate) struct FileImportStrategy {
    from_file: PathBuf,
}

#[derive(Clone, Debug)]
pub(crate) struct StdinImportStrategy {}

impl ImportStrategy for SomeImportStrategy {
    fn import(self) -> Result<Namespace> {
        match self {
            SomeImportStrategy::FromFile(fis) => fis.import(),
            SomeImportStrategy::FromPostgres(pis) => pis.import(),
            SomeImportStrategy::StdinImportStrategy(sis) => sis.import(),
        }
    }
}

impl ImportStrategy for FileImportStrategy {
    fn import(self) -> Result<Namespace> {
        let buff = std::fs::read_to_string(self.from_file)?;
        let collections: Map<String, Value> = serde_json::from_str(&buff)?;
        ns_from_json_object(collections)
    }
}

impl ImportStrategy for PostgresImportStrategy {
    fn import(self) -> Result<Namespace> {
        unimplemented!("Postgres is not supported yet")
    }
}

impl ImportStrategy for StdinImportStrategy {
    fn import(self) -> Result<Namespace> {
        let mut buff = String::new();
        std::io::stdin().read_to_string(&mut buff)?;
        let collections: Map<String, Value> = serde_json::from_str(&buff)?;
        ns_from_json_object(collections)
    }
}

fn ns_from_json_object(collections: Map<String, Value>) -> Result<Namespace> {
    let mut ns = Namespace::default();
    for (name, collection) in collections {
        let name = &Name::from_str(&name)?;
        match &collection {
            Value::Array(values) => {
                ns.create_collection(
                    name,
                    values.get(0).ok_or(anyhow!(
                        "Collection `{}` is empty. Failed to instantiate.",
                        name
                    ))?,
                )?;
                ns.try_update(name, &collection)?;
            }
            unacceptable_variant => {
                return Err(anyhow!(
                    "Was expecting a collection, instead got `{}`",
                    unacceptable_variant
                ))
            }
        }
    }
    Ok(ns)
}
