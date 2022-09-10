#![feature(async_closure, map_first_last, box_patterns, error_iter, try_blocks)]
#![allow(type_alias_bounds)]

#[macro_export]
macro_rules! derive_generator {
    {
        yield $yield:ty,
        return $return:ty,
        $vis:vis enum $id:ident {
            $(
                $variant:ident($inner:ty$(,)?)$(,)?
            )*
        }
    } => {
        $vis enum $id {
            $($variant($inner),)*
        }

        impl ::synth_gen::prelude::Generator for $id {
            type Yield = $yield;

            type Return = $return;

            fn next<R: Rng>(&mut self, rng: &mut R) -> ::synth_gen::prelude::GeneratorState<Self::Yield, Self::Return> {
                match self {
                    $(
                        Self::$variant(inner) => inner.next(rng),
                    )*
                }
            }
        }
    };
    {
        yield $yield:ty,
        return $return:ty,
        $vis:vis struct $id:ident($inner:ty);
    } => {
        $vis struct $id($inner);

        impl ::synth_gen::prelude::Generator for $id {
            type Yield = $yield;

            type Return = $return;

            fn next<R: Rng>(&mut self, rng: &mut R) -> ::synth_gen::prelude::GeneratorState<Self::Yield, Self::Return> {
                self.0.next(rng)
            }
        }
    }
}

#[macro_export]
macro_rules! schema {
    {
        $($inner:tt)*
    } => {
        serde_json::from_value::<$crate::schema::Content>(serde_json::json!($($inner)*))
            .expect("could not deserialize value into a schema")
    }
}

#[macro_export]
macro_rules! generator {
    { $($inner:tt)* } => {
        try_generator!($($inner)*).expect("could not compile the schema")
    }
}

#[macro_export]
macro_rules! try_generator {
    { $($inner:tt)* } => {
        $crate::Graph::from_content(&schema!($($inner)*))
    }
}

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

pub mod db_utils;
pub use db_utils::DataSourceParams;

#[macro_use]
pub mod schema;
pub use schema::{Content, Namespace};

pub mod graph;
pub use graph::{Graph, Value};

pub mod compile;
pub use compile::{Compile, Compiler};

#[cfg(test)]
pub mod tests {
    use rand::{rngs::StdRng, SeedableRng};
    use synth_gen::prelude::Generator;

    pub fn rng() -> StdRng {
        StdRng::seed_from_u64(0)
    }

    pub fn complete<G: Generator>(mut generator: G) -> G::Return {
        generator.complete(&mut rng())
    }

    pub fn complete_once<G: Generator>(generator: &mut G) -> G::Return {
        generator.complete(&mut rng())
    }
}
