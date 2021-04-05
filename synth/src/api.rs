use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use tide::{Body, Request, Response, Result as TideResult, Server, StatusCode};

use crate::{
    daemon::{
        DeleteCollectionRequest, DeleteNamespaceRequest, DeleteNamespaceRequestBody,
        DeleteOverrideRequest, DeleteOverrideRequestBody, GetDocumentsSampleRequest,
        GetDocumentsSampleRequestBody, GetNamespacesRequest, GetSchemaRequest,
        GetSchemaRequestQuery, PutDocumentsRequest, PutDocumentsRequestBody, PutOptionaliseRequest,
        PutOptionaliseRequestBody, PutOverrideRequest, PutOverrideRequestBody,
        PutOverrideRequestQuery, RollbackNamespaceRequest, RollbackNamespaceRequestBody,
    },
    error::UserError,
    Daemon,
};

use synth_core::Name;

#[inline]
fn get_optional(req: &Request<Api>, param: &str) -> TideResult<Option<Name>> {
    req.param(param)
        .and_then(|value| Ok(Some(FromStr::from_str(value)?)))
        .or_else(|e| match e.status() {
            StatusCode::NotFound => Ok(None),
            _ => Err(e.into()),
        })
}

#[derive(Clone)]
pub struct Api {
    daemon: Arc<Daemon>,
}

async fn delete_collection(req: Request<Api>) -> TideResult<Response> {
    let namespace: Name = FromStr::from_str(req.param("namespace")?)?;
    let collection: Name = FromStr::from_str(req.param("collection")?)?;

    let daemon_req = DeleteCollectionRequest {
        namespace,
        collection,
    };

    req.state().daemon.delete_collection(daemon_req)?;
    Ok(Response::new(StatusCode::Ok))
}

async fn delete_namespace(req: Request<Api>) -> TideResult<Response> {
    let namespace: Name = FromStr::from_str(req.param("namespace")?)?;

    let body: DeleteNamespaceRequestBody = req.query()?;

    let daemon_req = DeleteNamespaceRequest { namespace, body };

    req.state().daemon.delete_namespace(daemon_req)?;
    Ok(Response::new(StatusCode::Ok))
}

async fn rollback_namespace(req: Request<Api>) -> TideResult<Response> {
    let namespace: Name = FromStr::from_str(req.param("namespace")?)?;

    let body: RollbackNamespaceRequestBody = req.query()?;

    let daemon_req = RollbackNamespaceRequest { namespace, body };

    req.state().daemon.rollback_namespace(daemon_req)?;
    Ok(Response::new(StatusCode::Ok))
}

async fn put_documents(mut req: Request<Api>) -> TideResult<Response> {
    let namespace: Name = FromStr::from_str(req.param("namespace")?)?;
    let collection: Name = FromStr::from_str(req.param("collection")?)?;

    let body: PutDocumentsRequestBody = req.body_json().await?;

    #[cfg(debug)]
    trace!(
        "{op} namespace={namespace} collection={collection}",
        op = "PUT".bold(),
        namespace = namespace,
        collection = collection,
    );

    let daemon_req = PutDocumentsRequest {
        namespace,
        collection,
        body,
    };

    req.state().daemon.put_documents(daemon_req)?;
    Ok(Response::new(StatusCode::Ok))
}

async fn get_documents(req: Request<Api>) -> TideResult<Response> {
    let namespace: Name = FromStr::from_str(req.param("namespace")?)?;
    let collection: Option<Name> = get_optional(&req, "collection")?;

    let body: GetDocumentsSampleRequestBody = req.query()?;

    let daemon_req = GetDocumentsSampleRequest {
        namespace,
        collection,
        body,
    };

    req.state()
        .daemon
        .sample_documents(daemon_req)
        .map_err(|err| err.into())
        .and_then(|resp| {
            Body::from_json(&resp).map(|body| Response::builder(200).body(body).build())
        })
}

async fn get_namespaces(req: Request<Api>) -> TideResult<Response> {
    let daemon_req = GetNamespacesRequest;

    let resp = req.state().daemon.get_namespaces(daemon_req)?;
    Ok(Response::builder(StatusCode::Ok)
        .body(serde_json::to_value(resp).map_err(|e| {
            failed!(target: Release, Serialization => "failed to serialize namespaces").context(e)
        })?)
        .build())
}

async fn delete_override(req: Request<Api>) -> TideResult<Response> {
    let namespace: Name = FromStr::from_str(req.param("namespace")?)?;

    let body: DeleteOverrideRequestBody = req.query()?;

    let daemon_req = DeleteOverrideRequest { namespace, body };

    req.state().daemon.delete_override(daemon_req)?;

    Ok(Response::new(StatusCode::Ok))
}

async fn put_override(mut req: Request<Api>) -> TideResult<Response> {
    let namespace: Name = FromStr::from_str(req.param("namespace")?)?;

    let query: PutOverrideRequestQuery = req.query()?;

    let body: PutOverrideRequestBody = req.body_json().await?;

    #[cfg(debug)]
    trace!(
        "{op} namespace={namespace} override={body_str}",
        op = "PUT".bold(),
        namespace = namespace,
        body_str = serde_json::to_string(&body).unwrap_or("{{ INVALID }}".to_string())
    );

    let daemon_req = PutOverrideRequest {
        namespace,
        query,
        body,
    };

    req.state().daemon.put_override(daemon_req)?;
    Ok(Response::new(StatusCode::Ok))
}

async fn put_optionalise(mut req: Request<Api>) -> TideResult<Response> {
    let namespace: Name = FromStr::from_str(req.param("namespace")?)?;

    let body: PutOptionaliseRequestBody = req.body_json().await?;

    #[cfg(debug)]
    trace!(
        "{op} namespace={namespace} optionalise={body_str}",
        op = "PUT".bold(),
        namespace = namespace,
        body_str = serde_json::to_string(&body).unwrap_or("{{ INVALID }}".to_string())
    );

    let daemon_req = PutOptionaliseRequest { namespace, body };

    req.state().daemon.put_optionalise(daemon_req)?;
    Ok(Response::new(StatusCode::Ok))
}

async fn get_schema(req: Request<Api>) -> TideResult<Response> {
    let namespace: Name = FromStr::from_str(req.param("namespace")?)?;

    let query: GetSchemaRequestQuery = req.query()?;

    #[cfg(debug)]
    trace!(
        "{op} namespace={namespace} collection={collection}",
        op = "GET".bold(),
        namespace = namespace,
        collection = collection
            .as_ref()
            .map(|n| n.to_string())
            .unwrap_or("{not specified}".to_string())
    );

    let daemon_req = GetSchemaRequest { namespace, query };

    let resp = req.state().daemon.get_schema(daemon_req)?;
    Ok(Response::builder(StatusCode::Ok)
        .body(serde_json::to_value(resp).map_err(|e| {
            failed!(target: Release, Serialization => "failed to serialize schema").context(e)
        })?)
        .build())
}

impl Api {
    pub fn new_server(daemon: Arc<Daemon>) -> Result<Server<Self>> {
        let api = Self { daemon };
        let mut server = Server::with_state(api);
        server.at("/").get(|req: Request<Api>| get_namespaces(req));

        server
            .at("/:namespace/:collection")
            .put(|req: Request<Api>| put_documents(req))
            .delete(|req: Request<Api>| delete_collection(req));

        server
            .at("/:namespace")
            .delete(|req: Request<Api>| delete_namespace(req));

        server
            .at("/:namespace/_rollback")
            .put(|req: Request<Api>| rollback_namespace(req));

        server
            .at("/:namespace/_sample")
            .get(|req: Request<Api>| get_documents(req));

        server
            .at("/:namespace/:collection/_sample")
            .get(|req: Request<Api>| get_documents(req));

        server
            .at("/:namespace/_override")
            .put(|req: Request<Api>| put_override(req))
            .delete(|req: Request<Api>| delete_override(req));

        server
            .at("/:namespace/_schema")
            .get(|req: Request<Api>| get_schema(req));

        server
            .at("/:namespace/_optionalise")
            .put(|req: Request<Api>| put_optionalise(req));

        server
            .with(tide::utils::After(|mut res: Response| async move {
                match res.take_error() {
                    Some(error) => {
                        let user_error = UserError::from(error.as_ref());

                        #[cfg(not(debug_assertions))]
                        match error.downcast::<crate::error::Error>() {
                            Ok(error) => {
                                if !error.sensitive() {
                                    error!(target: "remote", "{}", user_error);
                                } else {
                                    error!("A crate::Error was intercepted but has been redacted because it is sensitive");
                                }
                            },
                            Err(_) => {
                                error!("An error was intercepted but has been redacted because it's sensitivity is unknown");
                            }
                        }

                        #[cfg(debug_assertions)]
                        {
			    error!("{}", error);
			    if let Some(backtrace) = error.backtrace() {
				error!("backtrace: {}", backtrace);
			    }
			}

                        Ok(user_error.into())
                    }
                    None => Ok(res),
                }
            }));

        #[cfg(debug_assertions)]
        server.with(tide::utils::Before(|req: Request<Api>| async move {
            use colored::Colorize;
            trace!(
                "{op} url={url} len={len}",
                op = req.method().to_string().bold(),
                url = req.url(),
                len = req
                    .len()
                    .map(|v| v.to_string())
                    .unwrap_or("unknown".to_string())
            );
            req
        }));

        Ok(server)
    }
}
