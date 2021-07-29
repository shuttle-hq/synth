use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use rand::SeedableRng;
use serde_json::{Map, Value};
use synth_core::Graph;
use synth_core::{Name, Namespace};
use synth_gen::prelude::*;

pub(crate) struct Sampler<'r> {
    namespace: &'r Namespace,
}

impl<'r> Sampler<'r> {
    pub fn new(namespace: &'r Namespace) -> Self {
        Self { namespace }
    }

    fn sampler_progress_bar(target: u64) -> ProgressBar {
        let bar = ProgressBar::new(target as u64);
        let style = ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {wide_bar} {pos}/{len} generated ({eta} remaining)");
        bar.set_style(style);
        bar
    }

    pub(crate) fn sample_seeded(
        &self,
        collection: Option<Name>,
        target: usize,
        seed: u64,
    ) -> Result<Value> {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        let graph = Graph::from_namespace(&self.namespace)?;
        let mut model = graph.into_iterator(&mut rng);

        let target = target as u64;
        let progress_bar = Self::sampler_progress_bar(target);
        let mut output = Map::new();

        while progress_bar.position() < target {
            let serializable = OwnedSerializable::new(&mut model);
            let mut is_empty = true;

            let as_value = match (serde_json::to_value(serializable), model.restart()) {
                (Ok(value), Ok(_)) => value,
                (_, Err(gen_err)) => return Err(gen_err.into()),
                (Err(ser_err), Ok(_)) => {
                    return Err(anyhow!("generated data is malformed: {}", ser_err))
                }
            };

            match as_value {
                Value::Object(map) => map.into_iter().try_for_each(|(name, value)| match value {
                    Value::Array(content) => {
                        is_empty &= content.is_empty();
                        progress_bar.inc(content.len() as u64);
                        output
                            .entry(name)
                            .or_insert_with(|| Value::Array(Vec::new()))
                            .as_array_mut()
                            .unwrap()
                            .extend(content);
                        Ok(())
                    }
                    _ => Err(failed!(
                        target: Release,
                        "namespace content was not an array (this should not happen)"
                    )),
                }),
                _ => Err(failed!(
                    target: Release,
                    "namespace was not an object (this should not happen)"
                )),
            }?;
            if is_empty {
                warn!(
                    "unable to generate {} values, only {} were generated",
                    target,
                    progress_bar.length()
                );
                break;
            }
        }

        progress_bar.finish();

        if let Some(name) = collection {
            let just = output.remove(&name.to_string()).ok_or(failed!(
                target: Release,
                "generated namespace does not have a collection {}",
                name
            ))?;
            Ok(just)
        } else {
            Ok(Value::Object(output))
        }
    }
}
