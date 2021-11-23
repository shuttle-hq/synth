use super::prelude::*;
use crate::Value;
use std::convert::{TryFrom, TryInto};
use std::path::PathBuf;
use uriparse::URI;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Hash)]
pub struct DatasourceContent {
    pub path: String,
}

impl Compile for DatasourceContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, mut _compiler: C) -> Result<Graph> {
        let iter = DataSourceParams {
            uri: URI::try_from(self.path.as_str())?,
            schema: None,
        }
        .try_into()?;

        Ok(Graph::Iter(IterNode { iter: iter }))
    }
}

pub struct DataSourceParams<'a> {
    pub uri: URI<'a>,
    pub schema: Option<String>, // PostgreSQL
}

impl TryFrom<DataSourceParams<'_>> for Box<dyn Iterator<Item = Value>> {
    type Error = anyhow::Error;

    fn try_from(params: DataSourceParams) -> Result<Self, Self::Error> {
        let scheme = params.uri.scheme().as_str().to_lowercase();

        let iter = match scheme.as_str() {
            "json" => {
                let path = PathBuf::from(params.uri.path().to_string());
                let file = std::fs::File::open(&path).or_else(|e| {
                    Err(failed_crate!(
                        target: Release,
                        "failed to open file: {}: {}",
                        path.display(),
                        e
                    ))
                })?;
                let arr: Vec<Value> = serde_json::from_reader(file).or_else(|e| {
                    Err(failed_crate!(
                        target: Release,
                        "failed to read file: {}: {}",
                        path.display(),
                        e
                    ))
                })?;

                arr.into_iter()
            }
            _ => {
                return Err(anyhow!(
                    "Datasource path scheme not recognised. Was expecting 'json'."
                ));
            }
        };

        Ok(Box::new(iter))
    }
}

#[cfg(test)]
mod tests {
    use super::{Content, DatasourceContent};
    use crate::compile::NamespaceCompiler;
    use std::path::PathBuf;

    #[test]
    fn compile() {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("src/schema/content/test.json");

        let content = DatasourceContent {
            path: format!("json:{}", p.display()),
        };

        let content = Content::Datasource(content);
        let compiler = NamespaceCompiler::new_flat(&content);
        compiler.compile().unwrap();
    }

    #[test]
    #[should_panic(expected = "scheme not recognised")]
    fn compile_unsupported_scheme() {
        let content = DatasourceContent {
            path: "mysql:".to_string(),
        };

        let content = Content::Datasource(content);
        let compiler = NamespaceCompiler::new_flat(&content);
        compiler.compile().unwrap();
    }

    #[test]
    #[should_panic(expected = "failed to open file: missing.json: No such file")]
    fn compile_file_not_found() {
        let content = DatasourceContent {
            path: "json:missing.json".to_string(),
        };

        let content = Content::Datasource(content);
        let compiler = NamespaceCompiler::new_flat(&content);
        compiler.compile().unwrap();
    }

    #[test]
    #[should_panic(expected = "failed to read file: ")]
    fn compile_not_array() {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("src/schema/content/invalid.json");

        let content = DatasourceContent {
            path: format!("json:{}", p.display()),
        };

        let content = Content::Datasource(content);
        let compiler = NamespaceCompiler::new_flat(&content);
        compiler.compile().unwrap();
    }
}
