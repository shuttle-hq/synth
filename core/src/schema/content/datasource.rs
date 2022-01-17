use super::prelude::*;
use crate::{DataSourceParams, Value};
use anyhow::Error;
use async_std::task;
use sqlx::{postgres::PgPoolOptions, Executor};
use std::path::PathBuf;
use uriparse::URI;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Hash)]
pub struct DatasourceContent {
    pub path: String,
    #[serde(default)]
    pub cycle: bool,
    pub schema: Option<String>,
    pub query: Option<String>,
}

impl Compile for DatasourceContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, mut _compiler: C) -> Result<Graph> {
        let params = DataSourceParams {
            uri: URI::try_from(self.path.as_str())?,
            schema: self.schema.clone(),
        };
        let iter =
            get_iter(params, self.query.clone()).map(|i| -> Box<dyn Iterator<Item = Value>> {
                if !self.cycle {
                    Box::new(i)
                } else {
                    Box::new(i.cycle())
                }
            })?;

        Ok(Graph::Iter(IterNode { iter }))
    }
}

fn get_iter(
    params: DataSourceParams,
    query: Option<String>,
) -> Result<impl Iterator<Item = Value> + Clone, Error> {
    let scheme = params.uri.scheme().as_str().to_lowercase();

    let iter = match scheme.as_str() {
        "json" => {
            let path = PathBuf::from(params.uri.path().to_string());
            let file = std::fs::File::open(&path).map_err(|e| {
                failed_crate!(
                    target: Release,
                    "failed to open file: {}: {}",
                    path.display(),
                    e
                )
            })?;
            let arr: Vec<Value> = serde_json::from_reader(file).map_err(|e| {
                failed_crate!(
                    target: Release,
                    "failed to read file: {}: {}",
                    path.display(),
                    e
                )
            })?;

            arr.into_iter()
        }
        "postgres" | "postgresql" => {
            let uri = params.uri.to_string();
            let rows = task::block_on(get_postgres_values(&uri, params.schema, query))?;

            rows.into_iter()
        }
        _ => {
            return Err(anyhow!(
                "Datasource path scheme not recognised. Was expecting 'json' or 'postgres'."
            ));
        }
    };

    Ok(iter)
}

async fn get_postgres_values(
    uri: &str,
    schema: Option<String>,
    query: Option<String>,
) -> Result<Vec<Value>> {
    if let Some(query) = query {
        let schema = schema.unwrap_or_else(|| "public".to_string());
        let pool = PgPoolOptions::new()
            .after_connect(move |conn| {
                let schema = schema.clone();
                Box::pin(async move {
                    conn.execute(&*format!("SET search_path = '{}';", schema))
                        .await?;
                    Ok(())
                })
            })
            .connect(uri)
            .await?;

        let rows = sqlx::query_as::<_, Value>(&query).fetch_all(&pool).await?;

        Ok(rows)
    } else {
        Err(anyhow!(
            "`datasource` with a database URI is missing a query"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{Content, DatasourceContent, Generator, GeneratorState};
    use crate::compile::NamespaceCompiler;
    use rand::SeedableRng;
    use std::path::PathBuf;

    #[test]
    fn compile() {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("src/schema/content/test.json");

        let content = DatasourceContent {
            path: format!(
                "json:{}",
                p.into_os_string().into_string().unwrap().replace('\\', "/")
            ),
            cycle: false,
            schema: None,
            query: None,
        };

        let content = Content::Datasource(content);
        let compiler = NamespaceCompiler::new_flat(&content);
        compiler.compile().unwrap();
    }

    #[test]
    fn compile_not_cycle() {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("src/schema/content/test.json");

        let content = DatasourceContent {
            path: format!(
                "json:{}",
                p.into_os_string().into_string().unwrap().replace('\\', "/")
            ),
            cycle: false,
            schema: None,
            query: None,
        };

        let content = Content::Datasource(content);
        let compiler = NamespaceCompiler::new_flat(&content);
        let mut graph = compiler.compile().unwrap();

        let mut seed = rand::rngs::StdRng::seed_from_u64(5);

        // test.json only has 6 items
        for _ in 0..6 {
            assert!(matches!(
                graph.next(&mut seed),
                GeneratorState::Complete(Ok(_))
            ));
        }
        assert!(matches!(
            graph.next(&mut seed),
            GeneratorState::Complete(Err(_))
        ));
    }

    #[test]
    fn compile_cycle() {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("src/schema/content/test.json");

        let content = DatasourceContent {
            path: format!(
                "json:{}",
                p.into_os_string().into_string().unwrap().replace('\\', "/")
            ),
            cycle: true,
            schema: None,
            query: None,
        };

        let content = Content::Datasource(content);
        let compiler = NamespaceCompiler::new_flat(&content);
        let mut graph = compiler.compile().unwrap();

        let mut seed = rand::rngs::StdRng::seed_from_u64(5);

        // test.json only has 6 items, but cycle is true
        for _ in 0..60 {
            assert!(matches!(
                graph.next(&mut seed),
                GeneratorState::Complete(Ok(_))
            ));
        }
    }

    #[test]
    #[should_panic(expected = "scheme not recognised")]
    fn compile_unsupported_scheme() {
        let content = DatasourceContent {
            path: "mysql:".to_string(),
            cycle: false,
            schema: None,
            query: None,
        };

        let content = Content::Datasource(content);
        let compiler = NamespaceCompiler::new_flat(&content);
        compiler.compile().unwrap();
    }

    #[test]
    #[should_panic(expected = "failed to open file: missing.json: ")]
    fn compile_file_not_found() {
        let content = DatasourceContent {
            path: "json:missing.json".to_string(),
            cycle: false,
            schema: None,
            query: None,
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
            path: format!(
                "json:{}",
                p.into_os_string().into_string().unwrap().replace('\\', "/")
            ),
            cycle: false,
            schema: None,
            query: None,
        };

        let content = Content::Datasource(content);
        let compiler = NamespaceCompiler::new_flat(&content);
        compiler.compile().unwrap();
    }

    #[test]
    #[should_panic(expected = "`datasource` with a database URI is missing a query")]
    fn compile_postgres_missing_query() {
        let content = DatasourceContent {
            path: "postgres://postgres:password@localhost:5432".to_string(),
            cycle: false,
            schema: None,
            query: None,
        };

        let content = Content::Datasource(content);
        let compiler = NamespaceCompiler::new_flat(&content);
        compiler.compile().unwrap();
    }
}
