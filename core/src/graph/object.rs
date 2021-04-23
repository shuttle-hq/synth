use super::prelude::*;

use std::collections::BTreeMap;
use synth_gen::value::Map;

pub struct KeyValueOrNothing {
    inner: Concatenate<JustToken<String>, Graph>,
    p: f64,
    active: bool,
}

impl Generator for KeyValueOrNothing {
    type Yield = Token;

    type Return = Option<(String, Result<Value, Error>)>;

    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        if !self.active {
            if self.p >= rng.gen() {
                self.active = true;
                self.next(rng)
            } else {
                GeneratorState::Complete(None)
            }
        } else {
            let next = self.inner.next(rng);
            if next.is_complete() {
                self.active = false;
            }
            next.map_complete(|c| Some(c))
        }
    }
}

impl KeyValueOrNothing {
    pub fn new_with(key: &str, content: Graph, freq: f64) -> Self {
        Self {
            inner: content.with_key(key.to_string().yield_token()),
            p: freq,
            active: false,
        }
    }

    pub fn always(key: &str, content: Graph) -> Self {
        Self::new_with(key, content, 1.0)
    }

    pub fn sometimes(key: &str, content: Graph) -> Self {
        Self::new_with(key, content, 0.5)
    }
}

pub struct ObjectNode(Map<Chain<KeyValueOrNothing>>);

impl FromIterator<KeyValueOrNothing> for ObjectNode {
    fn from_iter<T: IntoIterator<Item = KeyValueOrNothing>>(iter: T) -> Self {
        Self(Chain::from_iter(iter).into_map(None))
    }
}

impl Generator for ObjectNode {
    type Yield = Token;

    type Return = Result<Value, Error>;

    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        self.0.next(rng).map_complete(|kv| {
            kv.into_iter()
                .filter_map(|m_kv| m_kv.map(|(k, vr)| vr.map(|v| (k, v))))
                .collect::<Result<BTreeMap<_, _>, Error>>()
                .map(|hm| hm.into())
        })
    }
}
