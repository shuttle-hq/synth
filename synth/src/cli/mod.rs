mod export;
mod import;
mod postgres;
mod stdf;
mod store;

use crate::cli::export::SomeExportStrategy;
use crate::cli::export::{ExportParams, ExportStrategy};
use crate::cli::import::ImportStrategy;
use crate::cli::import::SomeImportStrategy;
use crate::cli::store::Store;
use anyhow::{Context, Result};

use std::convert::TryFrom;
use std::fs::File;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

use synth_core::Name;

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

    pub async fn run(self) -> Result<()> {
        match self.args {
            CliArgs::Generate {
                ref namespace,
                ref collection,
                size,
                ref to,
            } => self.generate(namespace.clone(), collection.clone(), size, to.clone()),
            CliArgs::Import {
                ref namespace,
                ref collection,
                ref from,
            } => self.import(namespace.clone(), collection.clone(), from.clone()),
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
                let workspace_dir = ".synth";
                let base_path = std::fs::canonicalize(".")?;
                let result = std::fs::create_dir(workspace_dir).context(format!(
                    "Failed to initialize workspace at: {}",
                    base_path.join(workspace_dir).to_str().unwrap()
                ));
                let config_path = ".synth/config.toml";
                match result {
                    Ok(()) => {
                        File::create(config_path).context(format!(
                            "Failed to initialize workspace at: {}",
                            base_path.join(config_path).to_str().unwrap()
                        ))?;
                        Ok(())
                    }
                    Err(ref e)
                        if e.downcast_ref::<std::io::Error>().unwrap().kind()
                            == std::io::ErrorKind::AlreadyExists =>
                    {
                        File::create(config_path).context(format!(
                            "Failed to initialize workspace at: {}",
                            base_path.join(config_path).to_str().unwrap()
                        ))?;
                        Ok(())
                    }
                    _ => result,
                }
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
                let content = import_strategy
                    .unwrap_or_default()
                    .import_collection(&collection)?;
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

    fn generate(
        &self,
        ns_path: PathBuf,
        collection: Option<Name>,
        target: usize,
        to: Option<SomeExportStrategy>,
    ) -> Result<()> {
        if !self.workspace_initialised() {
            return Err(anyhow!(
                "Workspace has not been initialised. To initialise the workspace run `synth init`."
            ));
        }
        let namespace = self
            .store
            .get_ns(&ns_path)
            .context("Unable to open the namespace")?;

        let params = ExportParams {
            namespace,
            collection_name: collection,
            target,
        };

        to.unwrap_or_default()
            .export(params)
            .context(format!("At namespace {:?}", ns_path))
    }
}

#[derive(StructOpt)]
#[structopt(name = "synth", about = "synthetic data engine on the command line")]
pub(crate) enum CliArgs {
    #[structopt(about = "Initialise the workspace")]
    Init {},
    #[structopt(about = "Generate data from a namespace")]
    Generate {
        #[structopt(
            help = "the namespace directory from which to generate",
            parse(from_os_str)
        )]
        namespace: PathBuf,
        #[structopt(long, help = "the specific collection from which to generate")]
        collection: Option<Name>,
        #[structopt(long, help = "the number of samples", default_value = "1")]
        size: usize,
        #[structopt(long, help = "the number of samples")]
        to: Option<SomeExportStrategy>,
    },
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
}
