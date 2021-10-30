mod export;
mod import;
mod import_utils;
mod mongo;
mod mysql;
mod postgres;
mod stdf;
mod store;

use crate::cli::db_utils::DataSourceParams;
use crate::cli::export::{ExportParams, ExportStrategy};
use crate::cli::import::ImportStrategy;
use crate::cli::store::Store;
use crate::version::print_version_message;

use anyhow::{Context, Result};
use rand::RngCore;
use serde::Serialize;
use std::convert::{TryFrom, TryInto};
use std::path::PathBuf;
use std::process::exit;
use structopt::clap::AppSettings;
use structopt::StructOpt;
use synth_core::{graph::json, Name};
use uriparse::URI;

pub(crate) mod config;
mod db_utils;
#[cfg(feature = "telemetry")]
pub mod telemetry;

pub struct Cli {
    store: Store,
}

impl Cli {
    pub fn new() -> Result<Self> {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

        #[cfg(debug_assertions)]
        {
            let splash = crate::utils::splash::Splash::auto()?;
            log::debug!("{}", splash);
        }

        Ok(Self {
            store: Store::init()?,
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
            Args::Init { .. } => Ok(()),
            Args::Generate {
                namespace,
                collection,
                size,
                ref to,
                seed,
                random,
                schema,
            } => self.generate(
                namespace,
                collection,
                size,
                to,
                Self::derive_seed(random, seed)?,
                schema,
            ),
            Args::Import {
                namespace,
                collection,
                ref from,
                schema,
            } => self.import(namespace, collection, from, schema),
            #[cfg(feature = "telemetry")]
            Args::Telemetry(cmd) => self.telemetry(cmd),
            Args::Version => {
                print_version_message();
                // Exiting so we don't get the message twice
                exit(0);
            }
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
                    println!("Telemetry is disabled. To enable it run `synth telemetry enable`.");
                }
                Ok(())
            }
        }
    }

    fn import(
        self,
        path: PathBuf,
        collection: Option<Name>,
        from: &str,
        schema: Option<String>,
    ) -> Result<()> {
        // TODO: If ns exists and no collection: break
        // If collection and ns exists and collection exists: break

        let import_strategy: Box<dyn ImportStrategy> = DataSourceParams {
            uri: URI::try_from(from).with_context(|| format!("Parsing import URI '{}'", from))?,
            schema,
        }
        .try_into()?;

        if let Some(collection) = collection {
            if self.store.collection_exists(&path, &collection) {
                return Err(anyhow!("The collection `{}` already exists. Will not import into an existing collection.",Store::relative_collection_path(&path, &collection).display()));
            } else {
                let content = import_strategy.import_collection(&collection)?;
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
            let ns = import_strategy.import()?;
            self.store.save_ns_path(path, ns)?;
            Ok(())
        }
    }

    fn generate(
        self,
        ns_path: PathBuf,
        collection: Option<Name>,
        target: usize,
        to: &str,
        seed: u64,
        schema: Option<String>,
    ) -> Result<()> {
        let namespace = self.store.get_ns(ns_path.clone()).context(format!(
            "Unable to open the namespace \"{}\"",
            ns_path
                .to_str()
                .expect("The provided namespace is not a valid UTF-8 string")
        ))?;

        let export_strategy: Box<dyn ExportStrategy> = DataSourceParams {
            uri: URI::try_from(to).with_context(|| format!("Parsing generation URI '{}'", to))?,
            schema,
        }
        .try_into()?;

        let params = ExportParams {
            namespace,
            collection_name: collection,
            target,
            seed,
        };

        export_strategy
            .export(params)
            .with_context(|| format!("At namespace {:?}", ns_path))
    }
}

// The serialization of this enum is used for telemetry when synth panics and we want our logs to
// contain the command that caused the panic. When modifying this, pay attention to skip
// serialization of any privacy sensitive information.
#[derive(StructOpt, Serialize)]
#[structopt(
    name = "synth",
    about = "synthetic data engine on the command line",
    no_version,
    global_settings = &[AppSettings::DisableVersion])]
pub enum Args {
    #[structopt(about = "(DEPRECATED). For backward compatibility and is a no-op.")]
    Init {
        #[serde(skip)]
        init_path: Option<PathBuf>,
    },
    #[structopt(about = "Generate data from a namespace", alias = "gen")]
    Generate {
        #[structopt(
            help = "The namespace directory from which to read schema files",
            parse(from_os_str)
        )]
        #[serde(skip)]
        namespace: PathBuf,
        #[structopt(long, help = "The specific collection from which to generate")]
        #[serde(skip)]
        collection: Option<Name>,
        #[structopt(long, help = "the number of samples", default_value = "1")]
        size: usize,
        #[structopt(
            long,
            help = "The URI into which data will be generated. Can be a file-based URI scheme to output data to the filesystem or stdout ('json:' and 'jsonl:' allow outputting JSON and JSON Lines data respectively) or can be a database URI to write data directly to some database (supports Postgres, MongoDB, and MySQL). Defaults to writing JSON data to stdout.",
            default_value = "json:"
        )]
        #[serde(skip)]
        to: String,
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
        #[structopt(
            long,
            help = "(Postgres only) Specify the schema into which to generate. Defaults to 'public'."
        )]
        #[serde(skip)]
        schema: Option<String>,
    },
    #[structopt(about = "Import data from an external source")]
    Import {
        #[structopt(
            help = "The namespace directory into which to save imported schema files",
            parse(from_os_str)
        )]
        #[serde(skip)]
        namespace: PathBuf,
        #[structopt(
            long,
            help = "The name of a collection into which the data will be imported"
        )]
        #[serde(skip)]
        collection: Option<Name>,
        #[structopt(
            long,
            help = "The source URI from which to import data. Can be a file-based URI scheme to read data from a file or stdin ('json:' and 'jsonl:' allow reading JSON and JSON Lines data respectively) or can be a database URI to read data directly from some database (supports Postgres, MongoDB, and MySQL). Defaults to reading JSON data from stdin.",
            default_value = "json:"
        )]
        #[serde(skip)]
        from: String,
        #[structopt(
            long,
            help = "(Postgres only) Specify the schema from which to import. Defaults to 'public'."
        )]
        #[serde(skip)]
        schema: Option<String>,
    },
    #[cfg(feature = "telemetry")]
    #[structopt(about = "Toggle anonymous usage data collection")]
    Telemetry(TelemetryCommand),
    #[structopt(about = "Version information")]
    Version,
}

#[cfg(feature = "telemetry")]
#[derive(StructOpt, Serialize)]
pub enum TelemetryCommand {
    #[structopt(about = "Enable anonymous usage data collection")]
    Enable,
    #[structopt(about = "Disable anonymous usage data collection")]
    Disable,
    #[structopt(about = "Check telemetry status")]
    Status,
}

#[derive(Debug, Clone)]
pub enum DataFormat {
    Json,
    JsonLines { collection_field_name: String },
}

impl DataFormat {
    pub fn new(uri_scheme: &str, uri_query: &str) -> Self {
        match uri_scheme {
            "jsonl" => DataFormat::JsonLines {
                collection_field_name: querystring::querify(uri_query)
                    .into_iter()
                    .find_map(|(key, value)| (key == "collection_field_name").then(|| value))
                    .unwrap_or("type")
                    .to_string(),
            },
            _ => DataFormat::Json,
        }
    }
}

impl Default for DataFormat {
    fn default() -> Self {
        DataFormat::Json
    }
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
