#![feature(total_cmp)]

pub mod encode;
pub use encode::Encoder;

pub mod decode;
pub use decode::Decoder;

pub mod module;
pub use module::Module;

pub mod error;
pub use error::Error;

#[cfg(feature = "train")]
#[pyo3::proc_macro::pymodule]
fn semantic_detection(py: pyo3::Python, m: &pyo3::types::PyModule) -> pyo3::PyResult<()> {
    let dummy = module::python_bindings::bind(py)?;
    m.add_submodule(dummy)?;
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use fake::{Dummy, Fake};
    use rand::{rngs::StdRng, SeedableRng};

    use arrow::array::{ArrayRef, StringArray, StringBuilder};
    use arrow::record_batch::RecordBatch;

    use std::sync::Arc;

    pub fn rng() -> StdRng {
        <StdRng as SeedableRng>::seed_from_u64(0xAAAAAAAAAAAAAAAA)
    }

    pub fn string_array_of<F>(f: F, len: usize) -> StringArray
    where
        String: Dummy<F>,
    {
        let mut builder = StringBuilder::new(len);
        let mut rng = rng();
        (0..len)
            .into_iter()
            .try_for_each(|_| builder.append_option(f.fake_with_rng::<Option<String>, _>(&mut rng)))
            .unwrap();
        builder.finish()
    }

    pub fn record_batch_of_with_names<S, A, I>(iter: I) -> RecordBatch
    where
        I: IntoIterator<Item = (S, A)>,
        S: AsRef<str>,
        A: arrow::array::Array + 'static,
    {
        RecordBatch::try_from_iter(
            iter.into_iter()
                .map(|(idx, array)| (idx.as_ref().to_string(), Arc::new(array) as ArrayRef)),
        )
        .unwrap()
    }

    pub fn record_batch_of<A, I>(iter: I) -> RecordBatch
    where
        I: IntoIterator<Item = A>,
        A: arrow::array::Array + 'static,
    {
        record_batch_of_with_names(
            iter.into_iter()
                .enumerate()
                .map(|(k, v)| (k.to_string(), v)),
        )
    }
}
