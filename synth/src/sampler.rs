use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use rand::SeedableRng;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use synth_core::schema::ValueKindExt;
use synth_core::Graph;
use synth_core::{Name, Namespace};
use synth_gen::prelude::*;

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

    pub(crate) fn sample(self, collection_name: Option<Name>, target: usize) -> Result<Value> {
        self.sample_seeded(collection_name, target, 0)
    }

    pub(crate) fn sample_seeded(
        self,
        collection_name: Option<Name>,
        target: usize,
        seed: u64,
    ) -> Result<Value> {
        fn value_as_array(name: &str, value: Value) -> Result<Vec<Value>> {
            match value {
                Value::Array(vec) => Ok(vec),
                _ => {
                    return Err(
                        failed!(target: Release, Unspecified => "generated data for collection '{}' is not of JSON type 'array', it is of type '{}'", name, value.kind()),
                    )
                }
            }
        }

        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mut model = self.graph.try_aggregate();

        let mut generated = 0;
        let mut out = BTreeMap::new();

        let progress_bar = Self::sampler_progress_bar(target as u64);

        while generated < target {
            let start_of_round = generated;
            let serializable = OwnedSerializable::new(model.try_next_yielded(&mut rng)?);
            let mut value = match serde_json::to_value(&serializable)? {
                Value::Object(map) => map,
                _ => {
                    return Err(
                        failed!(target: Release, Unspecified => "generated data is not a namespace"),
                    )
                }
            };

            if let Some(name) = collection_name.as_ref() {
                let collection_value = value.remove(name.as_ref()).ok_or(failed!(
                    target: Release,
                    "generated namespace does not have a collection '{}'",
                    name
                ))?;
                let vec = value_as_array(name.as_ref(), collection_value)?;
                generated += vec.len();
                out.entry(name.to_string())
                    .or_insert_with(|| Vec::new())
                    .extend(vec);
            } else {
                value.into_iter().try_for_each(|(collection, value)| {
                    value_as_array(&collection, value).and_then(|vec| {
                        generated += vec.len();
                        out.entry(collection)
                            .or_insert_with(|| Vec::new())
                            .extend(vec);
                        Ok(())
                    })
                })?;
            }

            if generated == start_of_round {
                warn!(
                    "could not generate the required target number of samples of {}",
                    target
                );
                break;
            }

            progress_bar.set_position(generated as u64);
        }

        progress_bar.finish();

        let as_value = if let Some(name) = collection_name.as_ref() {
            let array = out.remove(name.as_ref()).unwrap_or_default();
            Value::Array(array)
        } else {
            out.into_iter()
                .map(|(collection, values)| (collection, Value::Array(values)))
                .collect::<Map<String, Value>>()
                .into()
        };

        Ok(as_value)
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
