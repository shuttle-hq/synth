use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use rand::SeedableRng;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use synth_core::graph::json::synth_val_to_json;
use synth_core::{Content, Graph, Value};
use synth_gen::prelude::*;

pub(crate) struct Sampler {
    graph: Graph,
}

impl Sampler {
    pub(crate) fn sample_seeded(
        self,
        collection_name: Option<String>,
        target: usize,
        seed: u64,
    ) -> Result<SamplerOutput> {
        let rng = rand::rngs::StdRng::seed_from_u64(seed);
        let sample_strategy = SampleStrategy {
            collection_name,
            target,
        };
        sample_strategy.sample(self.graph, rng)
    }
}

impl TryFrom<&Content> for Sampler {
    type Error = anyhow::Error;
    fn try_from(ns: &Content) -> Result<Self> {
        Ok(Self {
            graph: Graph::try_from(ns)?,
        })
    }
}

#[derive(Clone)]
pub(crate) enum SamplerOutput {
    Namespace(Vec<(String, Value)>),
    Collection(String, Value),
}

impl SamplerOutput {
    pub(crate) fn into_json(self) -> serde_json::Value {
        let as_synth = match self {
            Self::Namespace(key_values) => {
                let object = key_values
                    .into_iter()
                    .map(|(key, value)| (key, value))
                    .collect();
                Value::Object(object)
            }
            Self::Collection(_, value) => value,
        };
        synth_val_to_json(as_synth)
    }
}

fn sampler_progress_bar(target: u64) -> ProgressBar {
    let bar = ProgressBar::new(target as u64);
    let style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {wide_bar} {pos}/{len} generated ({eta} remaining)");
    bar.set_style(style);
    bar
}

struct SampleStrategy {
    target: usize,
    collection_name: Option<String>,
}

impl SampleStrategy {
    fn sample<R: Rng>(self, model: Graph, mut rng: R) -> Result<SamplerOutput> {
        let mut generated = 0;
        let mut out = BTreeMap::<String, Value>::new();
        let progress_bar = sampler_progress_bar(self.target as u64);

        let ordered: Vec<_> = model
            .iter_ordered()
            .map(|iter| iter.map(|s| s.to_string()).collect())
            .unwrap_or_else(Vec::new);

        let mut model = model.aggregate();

        while generated < self.target {
            // We populate `out` by walking through the collections in the generated
            // namespace. We also keep track of the number of `Values` generated
            // for the progress bar.
            let round_start = generated;
            let next = model.complete(&mut rng)?;
            as_object(next)?
                .into_iter()
                .for_each(|(collection, value)| match value {
                    Value::Array(elements) => {
                        generated += elements.len();

                        let entry = out
                            .entry(collection)
                            .or_insert_with(|| Value::Array(vec![]));

                        if let Value::Array(to_extend) = entry {
                            to_extend.extend(elements);
                        }
                    }
                    non_array => {
                        generated += 1;
                        //out[&collection] = non_array;
                        out.insert(collection, non_array);
                    }
                });
            progress_bar.set_position(generated as u64);

            if round_start == generated {
                warn!("could not generate {} values: try modifying the schema to generate more data instead of using the --size flag", self.target);
                break;
            }
        }

        progress_bar.finish_and_clear();

        let sampler_output = match self.collection_name {
            Some(collection_name) => {
                let val = out.remove(&collection_name).unwrap(); //TODO
                SamplerOutput::Collection(collection_name, val)
            }
            None => {
                let mut ordered_out = Vec::new();

                for name in ordered {
                    let value = out.remove(&name).unwrap();
                    ordered_out.push((name, value));
                }

                ordered_out.extend(out.into_iter());
                SamplerOutput::Namespace(ordered_out)
            }
        };

        Ok(sampler_output)
    }
}

fn as_object(sample: Value) -> Result<BTreeMap<String, Value>> {
    match sample {
        Value::Object(obj) => Ok(obj),
        other => Err(anyhow!(
            "Was expecting the top-level sample to be an object. Instead found {}",
            other.type_()
        )),
    }
}
