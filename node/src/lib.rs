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
    fn js_new(mut cx: FunctionContext) -> JsResult<JsValue> {
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
    fn new(content: &Content, seed: u64) -> anyhow::Result<Self> {
        let graph = NamespaceCompiler::new_flat(content).compile()?;
        let rng = StdRng::seed_from_u64(seed);
        let synced = Arc::new(Mutex::new(graph.into_iterator(rng)));

        Ok(JsSampler(synced))
    }

    fn new_js_result<'a>(
        mut cx: impl Context<'a>,
        content: &Content,
        seed: u64,
    ) -> JsResult<'a, JsValue> {
        JsSampler::new(content, seed)
            .map(|sampler| cx.boxed(sampler).upcast())
            .or_else(|e| to_upcasted_type_error(cx, e))
    }

    /*
     * new_sampler(content: Content, seed: number)
     */
    fn js_new(mut cx: FunctionContext) -> JsResult<JsValue> {
        let js_content = cx.argument::<JsBox<JsContent>>(0)?;
        let seed = cx.argument::<JsNumber>(1)?.value(&mut cx) as u64;
        JsSampler::new_js_result(cx, &js_content.0, seed)
    }

    fn js_new_random_seed(mut cx: FunctionContext) -> JsResult<JsValue> {
        let js_content = cx.argument::<JsBox<JsContent>>(0)?;
        let seed: u64 = rand::thread_rng().gen();
        JsSampler::new_js_result(cx, &js_content.0, seed)
    }

    /*
     * sampler_next(this: Sampler)
     */
    fn js_next(mut cx: FunctionContext) -> JsResult<JsValue> {
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
    cx.export_function("new_content", JsContent::js_new)?;
    cx.export_function("new_sampler", JsSampler::js_new)?;
    cx.export_function("new_sampler_random_seed", JsSampler::js_new_random_seed)?;
    cx.export_function("sampler_next", JsSampler::js_next)?;
    Ok(())
}
