use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use rand::SeedableRng;
use serde_json::{Map, Value as JsonValue};
use std::collections::{btree_map::Entry, BTreeMap};
use std::convert::{TryFrom, TryInto};
use synth_core::graph::{IntoCompleted, Value};
use synth_core::schema::ValueKindExt;
use synth_core::Graph;
use synth_core::{Name, Namespace};
use synth_gen::prelude::*;

pub type Samples = Vec<Value>;

pub(crate) struct Sampler {
    graph: Graph,
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
        collection_name: Option<Name>,
        target: usize,
        seed: u64,
    ) -> Result<Value> {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mut model = self.graph.aggregate();
        let sample_strategy = SampleStrategy::new(collection_name, target);


        sample_strategy.sample(model, rng)
    }
}

impl TryFrom<&Namespace> for Sampler {
    type Error = anyhow::Error;
    fn try_from(namespace: &Namespace) -> Result<Self> {
        Ok(Self {
            graph: Graph::from_namespace(namespace)?,
        })
    }
}

enum SampleStrategy {
    Namespace(NamespaceSampleStrategy),
    Collection(CollectionSampleStrategy),
}

impl SampleStrategy {
    fn new(collection_name: Option<Name>, target: usize) -> Self {
        match collection_name {
            None => SampleStrategy::Namespace(NamespaceSampleStrategy {
                target
            }),
            Some(name) => SampleStrategy::Collection(CollectionSampleStrategy {
                name,
                target,
            })
        }
    }

    fn sample<R: Rng>(&self, model: Aggregate<Graph>, rng: R) -> Result<Value> {
        match self {
            SampleStrategy::Namespace(nss) => nss.sample(model, rng),
            SampleStrategy::Collection(css) => css.sample(model, rng)
        }
    }
}

struct NamespaceSampleStrategy {
    target: usize,
}

impl NamespaceSampleStrategy {
    fn sample<R: Rng>(&self, mut model: Aggregate<Graph>, mut rng: R) -> Result<Value> {
        let mut generated = 0;
        let mut out = BTreeMap::new();
        let progress_bar = sampler_progress_bar(self.target as u64);

        while generated < self.target {
            let next = model.complete(&mut rng)?;
            as_object(next)?
                .into_iter()
                .try_for_each(|(collection, value)| {
                    as_array(&collection, value)
                        .map(|vec| {
                            generated += vec.len();
                            match out.entry(collection) {
                                Entry::Vacant(e) => {
                                    e.insert(Value::Array(vec));
                                }
                                Entry::Occupied(mut o) => {
                                    match o.get_mut() {
                                        Value::Array(arr) => arr.extend(vec),
                                        _ => unreachable!("This is never not an array.")
                                    }
                                }
                            }
                        })
                })?;
            progress_bar.set_position(generated as u64);
        }
        Ok(Value::Object(out))
    }
}

struct CollectionSampleStrategy {
    name: Name,
    target: usize,
}

impl CollectionSampleStrategy {
    fn sample<R: Rng>(&self, mut model: Aggregate<Graph>, mut rng: R) -> Result<Value> {
        let mut out = vec![];
        let mut generated = 0;
        let progress_bar = sampler_progress_bar(self.target as u64);

        while generated < self.target {
            let next = model.complete(&mut rng)?;
            let collection_value = as_object(next)?.remove(self.name.as_ref()).ok_or_else(|| {
                anyhow!("generated namespace does not have a collection '{}'", self.name)
            })?;
            match collection_value {
                Value::Array(vec) => {
                    generated += vec.len();
                    out.extend(vec);
                }
                other => return Err(anyhow!("Was expecting the sampled collection to be an array. Instead found {}", other.type_()))
            }
            progress_bar.set_position(generated as u64);
        }

        Ok(Value::Array(out))
    }
}


fn as_object(sample: Value) -> Result<BTreeMap<String, Value>> {
    match sample {
        Value::Object(obj) => Ok(obj),
        other => Err(anyhow!("Was expecting the top-level sample to be an object. Instead found {}", other.type_()))
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