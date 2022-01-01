mod csv;
mod export;
mod import;
mod import_utils;
mod json;
mod jsonl;
mod mongo;
mod mysql;
mod postgres;
mod store;

use crate::cli::export::{ExportParams, ExportStrategy};
use crate::cli::import::ImportStrategy;
use crate::cli::store::Store;
use crate::version::print_version_message;

use anyhow::{Context, Result};
use rand::RngCore;
use serde::Serialize;
use std::cell::Cell;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::iter::FromIterator;
use std::path::PathBuf;
use structopt::clap::AppSettings;
use structopt::StructOpt;
use synth_core::DataSourceParams;
use uriparse::URI;

pub(crate) mod config;

#[cfg(feature = "telemetry")]
use std::cell::RefCell;
#[cfg(feature = "telemetry")]
use std::rc::Rc;
#[cfg(feature = "telemetry")]
pub mod telemetry;

#[cfg(feature = "telemetry")]
use telemetry::{TelemetryContext, TelemetryExportStrategy};

pub struct Cli {
    store: Store,
    export_strategy: Cell<Option<Box<dyn ExportStrategy>>>,
    #[cfg(feature = "telemetry")]
    telemetry_context: Rc<RefCell<TelemetryContext>>,
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
            export_strategy: Default::default(),
            #[cfg(feature = "telemetry")]
            telemetry_context: Rc::new(RefCell::new(TelemetryContext::new())),
        })
    }

    #[cfg(feature = "telemetry")]
    pub fn get_telemetry_context(&self) -> TelemetryContext {
        self.telemetry_context.borrow().clone()
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

    pub async fn run(&self, args: Args) -> Result<()> {
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
                Ok(())
            }
        }
    }

    #[cfg(feature = "telemetry")]
    fn telemetry(&self, cmd: TelemetryCommand) -> Result<()> {
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
        &self,
        path: PathBuf,
        collection_name: Option<String>,
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

        if let Some(collection_name) = collection_name {
            if self.store.collection_exists(&path, &collection_name) {
                return Err(anyhow!(
                    "The collection `{}` already exists. Will not import into an existing collection.",
                    Store::relative_collection_path(&path, &collection_name).display()
                ));
            } else {
                let content = import_strategy.import_collection(&collection_name)?;
                self.store
                    .save_collection_path(&path, collection_name, content)?;

                #[cfg(feature = "telemetry")]
                self.telemetry_context.borrow_mut().set_num_collections(1);

                Ok(())
            }
        } else if self.store.ns_exists(&path) {
            Err(anyhow!(
                "The directory at `{}` already exists. Will not import into an existing directory.",
                path.display()
            ))
        } else {
            let ns = import_strategy.import_namespace()?;

            #[cfg(feature = "telemetry")]
            TelemetryExportStrategy::fill_telemetry_pre(
                Rc::clone(&self.telemetry_context),
                &ns,
                collection_name,
                path.clone(),
            )?;

            self.store.save_ns_path(path, ns)?;

            Ok(())
        }
    }

    fn generate(
        &self,
        ns_path: PathBuf,
        collection_name: Option<String>,
        target: usize,
        to: &str,
        seed: u64,
        schema: Option<String>,
    ) -> Result<()> {
        let namespace = self.store.read_ns(ns_path.clone()).context(format!(
            "Unable to open the namespace \"{}\"",
            ns_path
                .to_str()
                .expect("The provided namespace is not a valid UTF-8 string")
        ))?;

        self.export_strategy.set(Some(
            DataSourceParams {
                uri: URI::try_from(to)
                    .with_context(|| format!("Parsing generation URI '{}'", to))?,
                schema,
            }
            .try_into()?,
        ));

        #[cfg(feature = "telemetry")]
        self.set_telemetry_export_strategy();

        let params = ExportParams {
            namespace,
            collection_name,
            target,
            seed,
            ns_path: ns_path.clone(),
        };

        self.export_strategy
            .take()
            .unwrap()
            .export(params)
            .with_context(|| format!("At namespace {:?}", ns_path))?;

        Ok(())
    }

    #[cfg(feature = "telemetry")]
    fn set_telemetry_export_strategy(&self) {
        let delegate = self.export_strategy.take();
        self.export_strategy
            .set(Some(Box::new(TelemetryExportStrategy::new(
                delegate.unwrap(),
                Rc::clone(&self.telemetry_context),
            ))));
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
        collection: Option<String>,
        #[structopt(long, help = "the number of samples", default_value = "1")]
        size: usize,
        #[structopt(
            long,
            help = "The URI into which data will be generated. Can be a file-based URI scheme to output data to the filesystem or stdout ('json:', 'jsonl:' and 'csv:' allow outputting JSON, JSON Lines and CSV data respectively) or can be a database URI to write data directly to some database (supports Postgres, MongoDB, and MySQL). Defaults to writing JSON data to stdout. [example: jsonl:/tmp/generation_output]",
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
        collection: Option<String>,
        #[structopt(
            long,
            help = "The source URI from which to import data. Can be a file-based URI scheme to read data from a file or stdin ('json:', 'jsonl:' and 'csv:' allow reading JSON, JSON Lines and CSV data respectively) or can be a database URI to read data directly from some database (supports Postgres, MongoDB, and MySQL). Defaults to reading JSON data from stdin. [example: jsonl:/tmp/test_data_input]",
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

fn map_from_uri_query<'a>(query_opt: Option<&'a uriparse::Query<'a>>) -> HashMap<&'a str, &'a str> {
    let query_str = query_opt.map(uriparse::Query::as_str).unwrap_or_default();

    HashMap::<_, _>::from_iter(querystring::querify(query_str).into_iter())
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
