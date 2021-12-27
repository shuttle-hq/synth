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

#[derive(Clone)]
pub(crate) enum SamplerOutput {
    Namespace(Vec<(String, Vec<Value>)>),
    Collection(String, Vec<Value>),
}

impl SamplerOutput {
    pub(crate) fn into_json(self) -> serde_json::Value {
        let as_synth = match self {
            Self::Namespace(key_values) => {
                let object = key_values
                    .into_iter()
                    .map(|(key, values)| (key, Value::Array(values)))
                    .collect();
                Value::Object(object)
            }
            Self::Collection(_, values) => Value::Array(values),
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

impl Sampler {
    pub(crate) fn sample_seeded(
        self,
        collection_name: Option<String>,
        target: usize,
        seed: u64,
    ) -> Result<SamplerOutput> {
        let rng = rand::rngs::StdRng::seed_from_u64(seed);
        let sample_strategy = SampleStrategy::new(collection_name, target);
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

enum SampleStrategy {
    Namespace(NamespaceSampleStrategy),
    Collection(CollectionSampleStrategy),
}

impl SampleStrategy {
    fn new(collection_name: Option<String>, target: usize) -> Self {
        match collection_name {
            None => SampleStrategy::Namespace(NamespaceSampleStrategy { target }),
            Some(name) => SampleStrategy::Collection(CollectionSampleStrategy { name, target }),
        }
    }

    fn sample<R: Rng>(self, model: Graph, rng: R) -> Result<SamplerOutput> {
        match self {
            SampleStrategy::Namespace(nss) => Ok(SamplerOutput::Namespace(nss.sample(model, rng)?)),
            SampleStrategy::Collection(css) => Ok(SamplerOutput::Collection(
                css.name.clone(),
                css.sample(model, rng)?,
            )),
        }
    }
}

struct NamespaceSampleStrategy {
    target: usize,
}

impl NamespaceSampleStrategy {
    fn sample<R: Rng>(self, model: Graph, mut rng: R) -> Result<Vec<(String, Vec<Value>)>> {
        let mut generated = 0;
        let mut out = BTreeMap::<String, Vec<Value>>::new();
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
                .try_for_each(|(collection, value)| {
                    as_array(&collection, value).map(|vec| {
                        generated += vec.len();
                        out.entry(collection).or_default().extend(vec);
                    })
                })?;
            progress_bar.set_position(generated as u64);
            if round_start == generated {
                warn!("could not generate {} values: try modifying the schema to generate more data instead of the --size flag", self.target);
                break;
            }
        }

        progress_bar.finish_and_clear();

        let mut ordered_out = Vec::new();

        for name in ordered {
            let value = out.remove(&name).unwrap();
            ordered_out.push((name, value));
        }

        ordered_out.extend(out.into_iter());

        Ok(ordered_out)
    }
}

struct CollectionSampleStrategy {
    name: String,
    target: usize,
}

impl CollectionSampleStrategy {
    fn sample<R: Rng>(self, model: Graph, mut rng: R) -> Result<Vec<Value>> {
        let mut out = vec![];
        let mut generated = 0;
        let progress_bar = sampler_progress_bar(self.target as u64);

        let mut model = model.aggregate();

        while generated < self.target {
            let round_start = generated;
            let next = model.complete(&mut rng)?;
            let collection_value = as_object(next)?.remove(&self.name).ok_or_else(|| {
                anyhow!(
                    "generated namespace does not have a collection '{}'",
                    self.name
                )
            })?;
            match collection_value {
                Value::Array(vec) => {
                    generated += vec.len();
                    out.extend(vec);
                }
                other => {
                    return Err(anyhow!(
                        "Was expecting the sampled collection to be an array. Instead found {}",
                        other.type_()
                    ))
                }
            }
            progress_bar.set_position(generated as u64);
            if round_start == generated {
                warn!("could not generate {} values for collection {}: try modifying the schema to generate more instead of using the --size flag", self.target, self.name);
                break;
            }
        }

        progress_bar.finish_and_clear();

        Ok(out)
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

fn as_array(name: &str, value: Value) -> Result<Vec<Value>> {
    match value {
        Value::Array(vec) => Ok(vec),
        _ => {
            return Err(
                anyhow!("generated data for collection '{}' is not of JSON type 'array', it is of type '{}'", name, value.type_()),
            );
        }
    }
}
