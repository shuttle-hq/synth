#![feature(map_first_last, box_patterns, concat_idents, error_iter)]
#![allow(type_alias_bounds)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate anyhow;

#[macro_use]
pub mod error;

pub mod cli;

pub mod datasource;
pub mod sampler;
pub mod utils;
pub mod version;

// Python Code
#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pymodule]
fn synth(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(generate, m)?)?;

    Ok(())
}

#[cfg(feature = "py")]
#[pyfunction]
pub fn generate() {
    todo!();
}
