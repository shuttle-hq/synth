#![feature(
    format_args_capture,
    async_closure,
    map_first_last,
    box_patterns,
    try_trait
)]
#![feature(error_iter)]
#![allow(type_alias_bounds)]
#![deny(warnings)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

//#[macro_use]
//extern crate lazy_static;

#[allow(unused_imports)]
#[macro_use]
extern crate serde_json;

use sysinfo::{System, SystemExt};

use structopt::StructOpt;

use colored::Colorize;

use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::{net::SocketAddr, str::FromStr};

use anyhow::Result;

#[cfg(feature = "python")]
use pyo3::{PyResult, Python};

#[macro_use]
mod error;

mod api;
pub use api::Api;

mod daemon;
use daemon::Daemon;

use crate::rlog::composite::CompositeLogger;
use crate::rlog::target::TargetLogger;
use crate::rlog::zenduty::ZenDuty;

mod cli;
mod rlog;

pub mod index;
use crate::cli::{Cli, CliArgs};
pub use index::Index;
use std::convert::TryFrom;

mod sampler;
pub mod store;

include!(concat!(env!("OUT_DIR"), "/meta.rs"));

struct DataDirectoryPath(PathBuf);

impl Default for DataDirectoryPath {
    fn default() -> Self {
        let path = env::current_dir()
            .expect("Failed to get current directory. Either the current directory does not exist or the user has insufficient permissions.")
            .join(".synth/");
        Self(path)
    }
}

impl ToString for DataDirectoryPath {
    fn to_string(&self) -> String {
        self.0.to_str().unwrap().to_string()
    }
}

impl FromStr for DataDirectoryPath {
    type Err = <PathBuf as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PathBuf::from_str(s).map(|pb| Self(pb))
    }
}

#[derive(StructOpt)]
pub(crate) enum Args {
    #[structopt(about = "Run Synth in daemon mode")]
    Serve(ServeArgs),
    #[structopt(flatten)]
    Cli(CliArgs),
}

#[derive(StructOpt)]
pub(crate) struct ServeArgs {
    #[structopt(short, long, default_value = "0.0.0.0:8182")]
    bind: SocketAddr,
    #[structopt(short, long, default_value)]
    data_directory: DataDirectoryPath,
    #[structopt(long)]
    zenduty: Option<String>,
}

struct Splash {
    python_ver: String,
    python_path: String,
    synth_ver: String,
    synth_ref: String,
    synth_rev: String,
    path: String,
    os: String,
    arch: String,
    mem: u64,
}

impl Splash {
    fn auto() -> Result<Self> {
        #[cfg(feature = "python")]
        let (python_ver, python_path) = {
            let gil = Python::acquire_gil();
            let py = gil.python();
            let sys = py.import("sys")?;
            let version = sys.get("version")?.extract::<String>()?.replace("\n", "");
            let path = sys.get("path")?.extract::<Vec<String>>()?.join(":");
            let out: PyResult<_> = Ok((version, path));
            out
        }?;

        #[cfg(not(feature = "python"))]
        let (python_ver, python_path) = ("disabled".bold().red().to_string(), String::new());

        let path = std::env::var("PATH").unwrap_or("unknown".to_string());

        let synth_ver = env!("CARGO_PKG_VERSION").to_string();

        let synth_ref = META_SHORTNAME.to_string();
        let synth_rev = META_OID.to_string();
        let os = META_OS.to_string();
        let arch = META_ARCH.to_string();

        let system = System::new_all();
        let mem = system.get_total_memory();

        Ok(Self {
            python_ver,
            python_path,
            synth_ver,
            synth_ref,
            synth_rev,
            path,
            os,
            arch,
            mem,
        })
    }
}

impl std::fmt::Display for Splash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "

version     = {synth_ver}
ref         = {synth_ref}
rev         = {synth_rev}
python      = {python_ver}
PYTHONPATH  = {python_path}
PATH        = {path}
target      = {os}
arch        = {arch}
threads     = {cpu}
mem         = {mem}
",
            synth_ver = self.synth_ver.blue().bold(),
            synth_ref = self.synth_ref.bold(),
            synth_rev = self.synth_rev.bold(),
            path = self.path.bold(),
            python_ver = self.python_ver.bold(),
            python_path = self.python_path,
            arch = self.arch.bold(),
            os = self.os.bold(),
            mem = self.mem,
            cpu = num_cpus::get()
        )?;
        Ok(())
    }
}

#[async_std::main]
async fn main() -> Result<()> {
    let args = Args::from_args();

    init_logger(&args);

    let splash = Splash::auto()?;
    debug!("{}", splash);

    match args {
        Args::Serve(sa) => serve_daemon(sa).await,
        Args::Cli(cli_args) => Cli::try_from(cli_args)?.run().await,
    }
}

async fn serve_daemon(args: ServeArgs) -> Result<()> {
    let daemon = Arc::new(Daemon::new(args.data_directory.0)?);

    let server = Api::new_server(daemon)?;
    eprintln!(
        "{} is listening on {}",
        "synth".bold(),
        args.bind.to_string()
    );

    ctrlc::set_handler(move || {
        // This is a hack which performs an ungraceful exit.
        // Some bug was introduced which results in signals from
        // the terminal being swallowed and not terminating the
        // application
        warn!("Received SIGINT! force exiting...");
        std::process::exit(0);
    })?;

    server.listen(args.bind).await?;

    Ok(())
}

fn init_logger(args: &Args) {
    let mut loggers = Vec::<Box<dyn log::Log>>::new();

    // Env logger
    let env_logger = env_logger::Builder::from_default_env().build();
    loggers.push(Box::new(env_logger));

    match args {
        Args::Serve(ServeArgs {
            zenduty: Some(api_key),
            ..
        }) => {
            let zen_logger = Box::new(TargetLogger::new(
                "remote".to_string(),
                ZenDuty::new(api_key.clone()),
            ));
            loggers.push(zen_logger);
        }
        _ => (),
    };

    CompositeLogger::init(loggers)
}
