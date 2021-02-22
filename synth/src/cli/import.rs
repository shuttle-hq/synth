use anyhow::{Result, Context};
use serde_json::Value;
use std::path::PathBuf;
use std::str::FromStr;
use synth_core::schema::{MergeStrategy, Namespace, Content, OptionalMergeStrategy};

pub(crate) trait ImportStrategy: Sized {
    fn import(self) -> Result<Namespace> {
        ns_from_value(self.into_value()?)
    }
    fn import_collection(self) -> Result<Content> {
        collection_from_value(&self.into_value()?)
    }
    fn into_value(self) -> Result<Value>;
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
    fn into_value(self) -> Result<Value> {
        match self {
            SomeImportStrategy::FromFile(fis) => fis.into_value(),
            SomeImportStrategy::FromPostgres(pis) => pis.into_value(),
            SomeImportStrategy::StdinImportStrategy(sis) => sis.into_value(),
        }
    }
}

impl ImportStrategy for FileImportStrategy {
    fn into_value(self) -> Result<Value> {
        Ok(serde_json::from_reader(std::fs::File::open(
            self.from_file,
        )?)?)
    }
}

impl ImportStrategy for PostgresImportStrategy {
    fn into_value(self) -> Result<Value> {
        unimplemented!("Postgres is not supported yet")
    }
}

impl ImportStrategy for StdinImportStrategy {
    fn into_value(self) -> Result<Value> {
        Ok(serde_json::from_reader(std::io::stdin())?)
    }
}

fn collection_from_value(value: &Value) -> Result<Content> {
    match value {
        Value::Array(values) => {
            let fst = values.get(0).unwrap_or(&Value::Null);
            let mut as_content = Namespace::collection(&fst);
            OptionalMergeStrategy.try_merge(&mut as_content, value)?;
            Ok(as_content)
        }
        unacceptable => Err(anyhow!(
            "Was expecting a collection, instead got `{}`",
            unacceptable
        )),
    }
}

fn ns_from_value(value: Value) -> Result<Namespace> {
    match value {
        Value::Object(object) => object
            .into_iter()
            .map(|(name, value)| {
                collection_from_value(&value)
                    .and_then(|content| Ok((name.parse()?, content)))
                    .context(anyhow!("While importing the collection `{}`", name))
            })
            .collect(),
        unacceptable => Err(anyhow!(
            "Was expecting an object, instead got `{}`",
            unacceptable
        )),
    }
}
