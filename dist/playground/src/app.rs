use crate::prelude::*;

use tide::{
    http::headers::HeaderValue,
    security::{CorsMiddleware, Origin},
    Body, Request, Response,
};

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
        let mut graph = NamespaceCompiler::new(&body).compile()?;
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

async fn put_compile(mut req: Request<State>) -> tide::Result {
    let query: CompileRequestQuery = req.query()?;
    let body: Content = match req.body_json().await {
        Ok(body) => body,
        Err(e) => {
            let error = ErrorResponseBody {
                kind: "schema",
                text: Some(e.to_string()),
            };
            let response = Response::builder(422)
                .body(Body::from_json(&error)?)
                .build();
            return Ok(response);
        }
    };
    info!(
        "compile request with query={:?} for content of kind {}",
        query,
        body.kind()
    );
    match req.state().compile_and_generate(body, query.size) {
        Ok(as_ser) => {
            let resp = Response::builder(200)
                .body(Body::from_json(&as_ser)?)
                .build();
            Ok(resp)
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
                let resp = Response::builder(400).body(Body::from_json(&body)?).build();
                Ok(resp)
            } else if let Some(app_err) = err.downcast_ref::<ErrorResponseBody>() {
                let resp = Response::builder(400)
                    .body(Body::from_json(&app_err)?)
                    .build();
                Ok(resp)
            } else {
                let resp = Response::builder(500)
                    .body(Body::from_string(err.to_string()))
                    .build();
                Ok(resp)
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

    let state = State { max_size };

    let mut app = tide::with_state(state);
    let mut root = app.at(&mount);

    root.put(put_compile);

    let cors = CorsMiddleware::new()
        .allow_methods(allow_methods.parse::<HeaderValue>().unwrap())
        .allow_credentials(true)
        .allow_origin(Origin::from(allow_origin))
        .allow_credentials(false);

    app.with(cors);

    let bind = SocketAddr::new(addr, port);
    app.listen(bind).await?;
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
