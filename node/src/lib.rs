use neon::prelude::*;
use neon_serde::from_value;

use synth_core::compile::NamespaceCompiler;
use synth_core::{Content, Graph};

use std::cell::RefCell;
use std::sync::Arc;

struct JsContent(Content);

impl Finalize for JsContent {}

fn new_content(mut cx: FunctionContext) -> JsResult<JsValue> {
    let schema = cx.argument::<JsValue>(0)?;

    match from_value(&mut cx, schema) {
        Ok(content) => {
            let boxed = cx.boxed(JsContent(content));
            Ok(boxed.upcast())
        }
        Err(e) => JsError::type_error(&mut cx, e.to_string()).map(|handle| handle.upcast()),
    }
}

type SharedGraph = Arc<RefCell<Graph>>;

struct JsGraph(SharedGraph);

unsafe impl std::marker::Send for JsGraph {} // TODO

impl Finalize for JsGraph {}

fn new_graph(mut cx: FunctionContext) -> JsResult<JsValue> {
    let js_content = cx.argument::<JsBox<JsContent>>(0)?;

    match NamespaceCompiler::new_flat(&js_content.0).compile() {
        Ok(graph) => {
            let boxed = cx.boxed(JsGraph(Arc::new(RefCell::new(graph))));
            Ok(boxed.upcast())
        }
        Err(e) => JsError::error(&mut cx, e.to_string()).map(|handle| handle.upcast()),
    }
}

struct JsSampler {
    graph: SharedGraph,
    seed: u64,
}

impl JsSampler {}

unsafe impl std::marker::Send for JsSampler {} // TODO

impl Finalize for JsSampler {}

fn new_sampler(mut cx: FunctionContext) -> JsResult<JsValue> {
    let js_graph = cx.argument::<JsBox<JsGraph>>(0)?;
    let seed = {
        if let Some(value) = cx.argument_opt(1) {
            let number = *value.downcast_or_throw::<JsNumber, FunctionContext>(&mut cx)?;
            number.value(&mut cx) as u64
        } else {
            0
        }
    };

    let boxed = cx.boxed(JsSampler {
        graph: Arc::clone(&js_graph.0),
        seed,
    });
    Ok(boxed.upcast())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("Content", new_content)?;
    cx.export_function("Compile", new_graph)?;
    cx.export_function("Sampler", new_sampler)?;
    Ok(())
}
