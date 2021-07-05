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

impl Sampler {
    fn sampler_progress_bar(target: u64) -> ProgressBar {
        let bar = ProgressBar::new(target as u64);
        let style = ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {wide_bar} {pos}/{len} generated ({eta} remaining)");
        bar.set_style(style);
        bar
    }

    pub(crate) fn sample_seeded(
        self,
        collection_name: Option<Name>,
        target: usize,
        seed: u64,
    ) -> Result<Samples> {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mut model = self.graph.aggregate();
        let mut generated = 0;
        let mut samples = vec![];

        let sample_strategy = match collection_name {
            None => Box::new(NamespaceSampleStrategy) as Box<dyn SampleStrategy>,
            Some(collection_name) => Box::new(CollectionSampleStrategy {
                name: collection_name
            }) as Box<dyn SampleStrategy>
        };

        while generated < target {
            let next = model.complete(&mut rng)?;
            let sample = sample_strategy.sample(as_object(next)?)?;
            samples.extend(sample);
        }

        Ok(samples)
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

fn as_object(sample: Value) -> Result<BTreeMap<String, Value>> {
    match sample {
        Value::Object(obj) => Ok(obj),
        other => Err(anyhow!("Was expecting the top-level sample to be an array. Instead found {}", other.type_()))
    }
}

trait SampleStrategy {
    fn sample(&self, sample: BTreeMap<String, Value>) -> Result<Samples>;
}

struct NamespaceSampleStrategy;

impl SampleStrategy for NamespaceSampleStrategy {
    fn sample(&self, sample: BTreeMap<String, Value>) -> Result<Samples> {
        todo!()
    }
}

struct CollectionSampleStrategy {
    name: Name,
}

impl SampleStrategy for CollectionSampleStrategy {
    fn sample(&self, mut sample: BTreeMap<String, Value>) -> Result<Samples> {
        let collection_value = sample.remove(self.name.as_ref()).ok_or_else(|| {
            anyhow!("generated namespace does not have a collection '{}'", self.name)
        })?;

        match collection_value {
            Value::Array(vec) => Ok(vec),
            other => Err(anyhow!("Was expecting the sampled collection to be an array. Instead found {}", other.type_()))
        }
    }
}
