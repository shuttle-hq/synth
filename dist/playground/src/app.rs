use crate::prelude::*;

use axum::{
    extract::Query,
    http,
    http::{HeaderValue, Method, StatusCode},
    response::IntoResponse,
    routing::put,
    Extension, Json, Router,
};
use tower_http::cors::CorsLayer;

use synth_core::{
    compile::NamespaceCompiler,
    error::{Error as SynthError, ErrorKind as SynthErrorKind},
    graph::{ArrayNode, Graph, RandomU64},
    schema::Content,
};
use synth_gen::prelude::*;

use super::ServeCmd;

#[derive(Clone)]
pub struct State {
    pub max_size: usize,
}

impl Default for State {
    fn default() -> Self {
        Self { max_size: 1024 }
    }
}

impl State {
    fn compile_and_generate(&self, body: Content, size: Option<u64>) -> Result<impl Serialize> {
        // Build the generator graph
        let mut graph = NamespaceCompiler::new_flat(&body).compile()?;
        if let Some(size) = size {
            let size = Graph::Number(RandomU64::constant(size).into()).into_size();
            graph = Graph::Array(ArrayNode::new_with(size, graph));
        }

        // Yield from the compiled graph. There is an upper bound to the
        // number of tokens the graph is allowed to generate to prevent
        // accidental or malicious blocking.
        let mut stream = Vec::new();
        loop {
            match graph.try_next(&mut OsRng) {
                GeneratorState::Yielded(yielded) => {
                    if stream.len() > self.max_size {
                        warn!("aborting: too large");
                        let text = "generated too many tokens: try generating less data by controlling (for example) the `length` parameter of arrays or using the `?size=` query parameter".to_string();
                        let body = ErrorResponseBody {
                            kind: "illegal",
                            text: Some(text),
                        };
                        return Err(Error::new(body));
                    }
                    stream.push(yielded);
                }
                GeneratorState::Complete(Err(err)) => {
                    warn!("aborting: generation error: {}", err);
                    return Err(err).context(anyhow!("while generating token {}", stream.len()));
                }
                GeneratorState::Complete(Ok(_)) => {
                    break;
                }
            }
        }

        Ok(OwnedSerializable::new(stream))
    }
}

#[derive(Debug, Deserialize)]
pub struct CompileRequestQuery {
    size: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponseBody {
    kind: &'static str,
    text: Option<String>,
}

impl std::fmt::Display for ErrorResponseBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}",
            self.kind,
            self.text.as_deref().unwrap_or("unknown")
        )
    }
}

impl std::error::Error for ErrorResponseBody {}

async fn put_compile(
    Extension(mut states): Extension<Arc<State>>,
    Json(body): Json<Content>,
    query: Query<CompileRequestQuery>,
) -> impl IntoResponse {
    let state = *states;
    let query: CompileRequestQuery = query.0;
    info!(
        "compile request with query={:?} for content of kind {}",
        query,
        body.kind()
    );
    match state.compile_and_generate(body, query.size) {
        Ok(as_ser) => {
            let resp = (StatusCode::OK, Json(&as_ser));
            return resp;
        }
        Err(err) => {
            if let Some(synth_err) = err.downcast_ref::<SynthError>() {
                let kind = match synth_err.kind {
                    SynthErrorKind::Compilation => "compilation",
                    SynthErrorKind::BadRequest => "schema",
                    _ => "unknown",
                };
                let body = ErrorResponseBody {
                    kind,
                    text: synth_err.msg.clone(),
                };
                let resp = (StatusCode::from_u16(400).unwrap(), Json(&body));
                return resp;
            } else if let Some(app_err) = err.downcast_ref::<ErrorResponseBody>() {
                let resp = (StatusCode::from_u16(400), Json(&app_err));
                return resp;
            } else {
                let resp = (StatusCode::from_u16(500), err.to_string());
                return resp;
            }
        }
    }
}

pub async fn serve(args: ServeCmd) -> Result<()> {
    debug!("serve args: {:?}", args);

    let ServeCmd {
        addr,
        port,
        mount,
        max_size,
        allow_methods,
        allow_origin,
    } = args;

    let state = Arc::new(State { max_size });

    let mut allowed_methods = Vec::new();

    let methods = [
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::HEAD,
        Method::OPTIONS,
        Method::CONNECT,
        Method::PATCH,
        Method::TRACE,
    ];
    for i in 0..8 {
        if allow_methods.contains(methods[i].as_str()) {
            allowed_methods.push(methods[i].clone());
        }
    }

    let app = Router::new()
        .route(&mount, put(put_compile))
        .layer(Extension(state))
        .layer(
            CorsLayer::new()
                .allow_methods(allowed_methods)
                .allow_credentials(true)
                .allow_origin(allow_origin.parse::<HeaderValue>().unwrap())
                .allow_credentials(false)
                .allow_headers(vec![http::header::CONTENT_TYPE]),
        );

    let bind = SocketAddr::new(addr, port);
    axum::Server::bind(&bind)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use std::fs::File;

    use serde_json::Value;

    #[test]
    fn compile_and_generate() -> Result<()> {
        let state = State::default();
        let mut f = File::open("tests/users.json")?;
        let content: Content = serde_json::from_reader(&mut f)?;

        let as_str = serde_json::to_string_pretty(&state.compile_and_generate(content, Some(10))?)?;
        let as_value: Value = serde_json::from_str(&as_str)?;
        match as_value {
            Value::Array(arr) if arr.len() == 10 => {
                let all_objects = arr.into_iter().all(|elt| matches!(elt, Value::Object(_)));
                assert!(all_objects);
            }
            _ => panic!("incorrectly generated: {}", as_str),
        }
        Ok(())
    }
}
