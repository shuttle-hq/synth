#![feature(
    format_args_capture,
    async_closure,
    map_first_last,
    box_patterns,
    try_trait,
    error_iter
)]
#![allow(type_alias_bounds)]
#![deny(warnings)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate lazy_static;

#[allow(unused_imports)]
#[macro_use]
extern crate serde_json;

#[macro_use]
pub mod error;
pub use error::Error;

pub mod index;
pub use index::Index;

#[macro_use]
pub mod schema;
pub use schema::{Content, Namespace, Name};

pub mod python;

pub mod gen;
//mod rlog;
pub mod store;
