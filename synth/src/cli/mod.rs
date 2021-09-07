mod export;
mod import;
mod import_utils;
mod mongo;
mod mysql;
mod postgres;
mod stdf;
mod store;

use crate::cli::export::SomeExportStrategy;
use crate::cli::export::{ExportParams, ExportStrategy};
use crate::cli::import::ImportStrategy;
use crate::cli::import::SomeImportStrategy;
use crate::cli::store::Store;

use anyhow::{Context, Result};

use std::path::PathBuf;
use structopt::StructOpt;

use rand::RngCore;

use synth_core::{Name, graph::json};

#[cfg(feature = "telemetry")]
pub mod telemetry;

pub struct Cli {
    store: Store
}

impl Cli {
    /// this is going to get confusing with `init` command
    pub fn new() -> Result<Self> {
        env_logger::init();

        #[cfg(debug_assertions)]
        {
            let splash = crate::utils::splash::Splash::auto()?;
            log::debug!("{}", splash);
        }

        Ok(Self {
            store: Store::init()?
        })
    }

    fn derive_seed(random: bool, seed: Option<u64>) -> Result<u64> {
        if random && seed.is_some() {
            return Err(anyhow!(
                "Cannot have the --random flag and --seed specified at the same time."
            ));
        }
        match random {
            true => Ok(rand::thread_rng().next_u64()),
            false => Ok(seed.unwrap_or(0)),
        }
    }

    pub async fn run(self, args: Args) -> Result<()> {
        match args {
            Args::Init { .. } => {
                Ok(())
            },
            Args::Generate {
                ref namespace,
                ref collection,
                size,
                ref to,
                seed,
                random,
            } => self.generate(
                namespace.clone(),
                collection.clone(),
                size,
                to.clone(),
                Self::derive_seed(random, seed)?,
            ),
            Args::Import {
                ref namespace,
                ref collection,
                ref from,
            } => self.import(namespace.clone(), collection.clone(), from.clone()),
            #[cfg(feature = "telemetry")]
            Args::Telemetry(cmd) => self.telemetry(cmd),
        }
    }

    #[cfg(feature = "telemetry")]
    fn telemetry(self, cmd: TelemetryCommand) -> Result<()> {
        match cmd {
            TelemetryCommand::Enable => telemetry::enable(),
            TelemetryCommand::Disable => telemetry::disable(),
            TelemetryCommand::Status => {
                if telemetry::is_enabled() {
                    println!("Telemetry is enabled. To disable it run `synth telemetry disable`.");
                } else {
                    println!(
                        "Telemetry is disabled. To enable it run `synth telemetry enable`."
                    );
                }
                Ok(())
            }
        }
    }

    fn import(
        self,
        path: PathBuf,
        collection: Option<Name>,
        import_strategy: Option<SomeImportStrategy>,
    ) -> Result<()> {
        // TODO: If ns exists and no collection: break
        // If collection and ns exists and collection exists: break
        if let Some(collection) = collection {
            if self.store.collection_exists(&path, &collection) {
                return Err(anyhow!(
                    "The collection `{}` already exists. Will not import into an existing collection.",
		    Store::relative_collection_path(&path, &collection).display()
		));
            } else {
                let content = import_strategy
                    .unwrap_or_default()
                    .import_collection(&collection)?;
                self.store
                    .save_collection_path(&path, collection, content)?;
                Ok(())
            }
        } else if self.store.ns_exists(&path) {
            Err(anyhow!(
                "The directory at `{}` already exists. Will not import into an existing directory.",
                path.display()
            ))
        } else {
            let ns = import_strategy.unwrap_or_default().import()?;
            self.store.save_ns_path(path, ns)?;
            Ok(())
        }
    }

    fn generate(
        self,
        ns_path: PathBuf,
        collection: Option<Name>,
        target: usize,
        to: Option<SomeExportStrategy>,
        seed: u64,
    ) -> Result<()> {
        let namespace = self
            .store
            .get_ns(ns_path.clone())
            .context("Unable to open the namespace")?;
        let params = ExportParams {
            namespace,
            collection_name: collection,
            target,
            seed,
        };

        to.unwrap_or_default()
            .export(params)
            .with_context(|| format!("At namespace {:?}", ns_path))
    }
}

#[derive(StructOpt)]
#[structopt(name = "synth", about = "synthetic data engine on the command line")]
pub enum Args {
    #[structopt(about = "(DEPRECATED). For backward compatibility and is a no-op.")]
    Init {
	init_path: Option<PathBuf>
    },
    #[structopt(about = "Generate data from a namespace", alias = "gen")]
    Generate {
        #[structopt(
            help = "The namespace directory from which to read schema files",
            parse(from_os_str)
        )]
        namespace: PathBuf,
        #[structopt(long, help = "The specific collection from which to generate")]
        collection: Option<Name>,
        #[structopt(long, help = "the number of samples", default_value = "1")]
        size: usize,
        #[structopt(
            long,
            help = "The sink into which to generate data. Can be a postgres uri, a mongodb uri. If not specified, data will be written to stdout"
        )]
        to: Option<SomeExportStrategy>,
        #[structopt(
            long,
            help = "an unsigned 64 bit integer seed to be used as a seed for generation"
        )]
        seed: Option<u64>,
        #[structopt(
            long,
            help = "generation will use a random seed - this cannot be used with --seed"
        )]
        random: bool,
    },
    #[structopt(about = "Import data from an external source")]
    Import {
        #[structopt(
            help = "The namespace directory into which to save imported schema files",
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
            help = "The source from which to import data. Can be a postgres uri, a mongodb uri or a path to a JSON file / directory. If not specified, data will be read from stdin"
        )]
        from: Option<SomeImportStrategy>,
    },
    #[cfg(feature = "telemetry")]
    #[structopt(about = "Toggle anonymous usage data collection")]
    Telemetry(TelemetryCommand),
}

#[cfg(feature = "telemetry")]
#[derive(StructOpt)]
pub enum TelemetryCommand {
    #[structopt(about = "Enable anonymous usage data collection")]
    Enable,
    #[structopt(about = "Disable anonymous usage data collection")]
    Disable,
    #[structopt(about = "Check telemetry status")]
    Status,
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_derive_seed() {
        assert_eq!(Cli::derive_seed(false, None).unwrap(), 0);
        assert_eq!(Cli::derive_seed(false, Some(5)).unwrap(), 5);
        assert!(Cli::derive_seed(true, Some(5)).is_err());
        assert!(Cli::derive_seed(true, None).is_ok());
    }
}
