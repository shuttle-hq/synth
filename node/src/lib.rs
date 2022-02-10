#![allow(clippy::new_ret_no_self)]

use neon::prelude::*;
use neon_serde::{from_value, to_value};

use rand::SeedableRng;
use synth_core::compile::NamespaceCompiler;
use synth_core::{Content, Graph};

use synth_gen::prelude::*;

use rand::rngs::StdRng;

use std::fmt::Display;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

struct JsContent(Content);

impl Finalize for JsContent {}

impl JsContent {
    /*
     * new_content(schema)
     */
    fn new(mut cx: FunctionContext) -> JsResult<JsValue> {
        let schema = cx.argument::<JsValue>(0)?;

        match from_value(&mut cx, schema) {
            Ok(content) => {
                let boxed = cx.boxed(JsContent(content));
                Ok(boxed.upcast())
            }
            Err(e) => to_upcasted_type_error(cx, e),
        }
    }
}

struct JsSampler(Arc<Mutex<Iterable<Graph, StdRng>>>);

unsafe impl std::marker::Send for JsSampler {} // TODO

impl Finalize for JsSampler {}

impl JsSampler {
    /*
     * new_sampler(content: Content, seed: number)
     */
    fn new(mut cx: FunctionContext) -> JsResult<JsValue> {
        let js_content = cx.argument::<JsBox<JsContent>>(0)?;
        let seed = cx.argument::<JsNumber>(1)?.value(&mut cx) as u64;

        match NamespaceCompiler::new_flat(&js_content.0).compile() {
            Ok(graph) => {
                let rng = StdRng::seed_from_u64(seed);
                let synced = Arc::new(Mutex::new(graph.into_iterator(rng)));
                let boxed = cx.boxed(JsSampler(synced));

                Ok(boxed.upcast())
            }
            Err(e) => to_upcasted_type_error(cx, e),
        }
    }

    /*
     * sampler_next(this: Sampler)
     */
    fn next(mut cx: FunctionContext) -> JsResult<JsValue> {
        let this = cx.argument::<JsBox<JsSampler>>(0)?;

        let mut iter_lock = this.0.lock().unwrap();

        match to_value(&mut cx, &OwnedSerializable::new(iter_lock.deref_mut())) {
            Ok(value) => match iter_lock.restart() {
                Ok(_) => Ok(value),
                Err(e) => to_upcasted_type_error(cx, e),
            },
            Err(e) => to_upcasted_type_error(cx, e),
        }
    }
}

fn to_upcasted_type_error<'a>(mut cx: impl Context<'a>, e: impl Display) -> JsResult<'a, JsValue> {
    JsError::type_error(&mut cx, e.to_string()).map(|handle| handle.upcast())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("new_content", JsContent::new)?;
    cx.export_function("new_sampler", JsSampler::new)?;
    cx.export_function("sampler_next", JsSampler::next)?;
    Ok(())
}
