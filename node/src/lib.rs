use neon::prelude::*;
use neon_serde::{from_value, to_value};

use synth_core::{Content, Graph};

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

struct JsGraph(Graph);

impl Finalize for JsGraph {}

fn new_graph(mut cx: FunctionContext) -> JsResult<JsValue> {
    let wrapped_content = cx.argument::<JsBox<JsContent>>(0)?;
    todo!()
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("Content", new_content)?;
    Ok(())
}
