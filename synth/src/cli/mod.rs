mod import;
mod store;

use crate::cli::import::ImportStrategy;
use crate::cli::import::SomeImportStrategy;
use crate::cli::store::Store;
use anyhow::Result;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use synth_core::graph::Graph;
use synth_core::schema::ValueKindExt;
use synth_core::Name;
use synth_gen::prelude::*;

/// synth init
/// synth import my_namespace from-file /a/path/to/some/file
/// synth generate my_namespace/

impl TryFrom<CliArgs> for Cli {
    type Error = anyhow::Error;
    fn try_from(args: CliArgs) -> Result<Self> {
        Cli::new(args)
    }
}

pub(crate) struct Cli {
    store: Store,
    args: CliArgs,
}

impl Cli {
    /// this is going to get confusing with `init` command
    pub(crate) fn new(args: CliArgs) -> Result<Self> {
        Ok(Self {
            store: Store::init()?,
            args,
        })
    }

    // Use this later. this probably introduces more complexity than it help rn
    // fn check_initialised<F: FnOnce() -> Result<()>>(&self, f: F) -> Result<()> {
    //     if !self.workspace_initialised() {
    //         return Err(anyhow!(
    //             "Workspace has not been initialised. Run `synth init` to initialise the workspace."
    //         ));
    //     }
    //     f()
    // }

    pub async fn run(&self) -> Result<()> {
        match self.args.clone() {
            CliArgs::Generate {
                namespace,
                collection,
                size,
            } => self.generate(namespace, collection, size),
            CliArgs::Import {
                namespace,
                collection,
                from,
            } => self.import(namespace, collection, from),
            CliArgs::Init {} => self.init(),
        }
    }

    fn init(&self) -> Result<()> {
        match self.workspace_initialised() {
            true => {
                println!("Workspace already initialised");
                std::process::exit(1)
            }
            false => {
                let _ = std::fs::create_dir(".synth"); // Ignore error here. It could be that the directory exists but not the config file
                File::create(".synth/config.toml")?;
                Ok(())
            }
        }
    }

    fn workspace_initialised(&self) -> bool {
        Path::new(".synth/config.toml").exists()
    }

    fn import(
        &self,
        path: PathBuf,
        collection: Option<Name>,
        import_strategy: Option<SomeImportStrategy>,
    ) -> Result<()> {
        if !self.workspace_initialised() {
            return Err(anyhow!(
                "Workspace has not been initialised. To initialise the workspace run `synth init`."
            ));
        }

        if !path.is_relative() {
            return Err(anyhow!(
		"The namespace path `{}` is absolute. Only paths relative to an initialised workspace root are accepted.",
		path.display()
	    ));
        }

        // TODO: If ns exists and no collection: break
        // If collection and ns exists and collection exists: break
        if let Some(collection) = collection {
            if self.store.collection_exists(&path, &collection) {
                return Err(anyhow!(
                    "The collection `{}` already exists. Will not import into an existing collection.",
		    Store::relative_collection_path(&path, &collection).display()
		));
            } else {
                let content = import_strategy.unwrap_or_default().import_collection()?;
                self.store
                    .save_collection_path(&path, collection, content)?;
                Ok(())
            }
        } else {
            if self.store.ns_exists(&path) {
                return Err(anyhow!(
                    "The namespace at `{}` already exists. Will not import into an existing namespace.",
                    path.display()
		));
            } else {
                let ns = import_strategy.unwrap_or_default().import()?;
                self.store.save_ns_path(path, ns)?;
                Ok(())
            }
        }
    }

    fn generate(&self, ns_path: PathBuf, collection: Option<Name>, target: usize) -> Result<()> {
        if !self.workspace_initialised() {
            return Err(anyhow!(
                "Workspace has not been initialised. To initialise the workspace run `synth init`."
            ));
        }
        let ns = self.store.get_ns(ns_path)?;

        let mut rng = rand::thread_rng();
        let mut model = Graph::from_namespace(&ns)?.aggregate();

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

        let mut generated = 0;

        let mut out = HashMap::new();

        while generated < target {
            let start_of_round = generated;
            let serializable = OwnedSerializable::new(model.try_next_yielded(&mut rng)?);
            let mut value = match serde_json::to_value(&serializable)? {
                Value::Object(map) => map,
                _ => {
                    return Err(
                        failed!(target: Release, Unspecified => "generated synthetic data is not a namespace"),
                    )
                }
            };

            if let Some(name) = collection.as_ref() {
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
        }

        let as_value = if let Some(name) = collection.as_ref() {
            let array = out.remove(name.as_ref()).unwrap_or_default();
            Value::Array(array)
        } else {
            out.into_iter()
                .map(|(collection, values)| (collection, Value::Array(values)))
                .collect::<Map<String, Value>>()
                .into()
        };

        println!("{}", as_value);

        Ok(())
    }
}

#[derive(StructOpt, Clone)]
#[structopt(name = "synth", about = "synthetic data engine on the command line")]
pub(crate) enum CliArgs {
    #[structopt(about = "Create a new empty workspace in the current working directory")]
    Init {},
    #[structopt(about = "Create a new namespace from existing JSON data")]
    Import {
        #[structopt(
            help = "The namespace directory into which to import",
            parse(from_os_str)
        )]
        namespace: PathBuf,
        #[structopt(
            long,
            help = "The name of a collection into which the data will be imported"
        )]
        collection: Option<Name>,
        #[structopt(
            long,
            help = "Path to a JSON file containing the data to import. If not specified, data will be read from stdin"
        )]
        from: Option<SomeImportStrategy>,
    },
    #[structopt(about = "Generate data for a namespace")]
    Generate {
        #[structopt(
            help = "The namespace directory from which to generate",
            parse(from_os_str)
        )]
        namespace: PathBuf,
        #[structopt(
            long,
            help = "The name of a collection for which to generate data. If not specified, will generate data for all collections in the namespace"
        )]
        collection: Option<Name>,
        #[structopt(long, default_value = "1")]
        size: usize,
    },
}
