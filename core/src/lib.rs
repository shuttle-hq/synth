#![feature(
    format_args_capture,
    async_closure,
    map_first_last,
    box_patterns,
    error_iter
)]
#![allow(type_alias_bounds)]
// #![deny(warnings)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate lazy_static;

#[allow(unused_imports)]
#[macro_use]
extern crate serde_json;

extern crate humantime_serde;

#[macro_use]
pub mod error;
pub use error::Error;

#[macro_use]
pub mod schema;
pub use schema::{Content, Name, Namespace};

pub mod graph;
pub use graph::Graph;

pub mod compile;
pub use compile::{Compile, Compiler};
