#![feature(format_args_capture, async_closure, map_first_last, box_patterns)]
#![feature(error_iter)]
#![allow(type_alias_bounds)]
#![deny(warnings)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate anyhow;

// #[macro_use]
// extern crate diesel;

// #[macro_use]
// extern crate diesel_migrations;

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
use std::{net::SocketAddr, str::FromStr};

use anyhow::Result;

#[macro_use]
mod error;

#[cfg(feature = "api")]
mod api;
#[cfg(feature = "api")]
pub use api::Api;

#[cfg(feature = "api")]
mod daemon;
#[cfg(feature = "api")]
use daemon::Daemon;

use crate::rlog::composite::CompositeLogger;

pub mod cli;
mod rlog;

#[cfg(feature = "api")]
pub mod index;
#[cfg(feature = "api")]
pub use index::Index;

use crate::cli::CliArgs;

mod sampler;
pub mod store;
mod datasource;

include!(concat!(env!("OUT_DIR"), "/meta.rs"));

pub struct DataDirectoryPath(PathBuf);

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
        PathBuf::from_str(s).map(Self)
    }
}

pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[derive(StructOpt)]
pub enum Args {
    #[cfg(feature = "api")]
    #[structopt(about = "Run Synth in daemon mode")]
    Serve(ServeArgs),
    #[structopt(flatten)]
    Cli(CliArgs),
}

#[derive(StructOpt)]
pub struct ServeArgs {
    #[structopt(short, long, default_value = "0.0.0.0:8182")]
    pub bind: SocketAddr,
    #[structopt(short, long, default_value)]
    pub data_directory: DataDirectoryPath,
    #[structopt(long)]
    pub zenduty: Option<String>,
}

pub struct Splash {
    synth_ver: String,
    synth_ref: String,
    synth_rev: String,
    path: String,
    os: String,
    arch: String,
    mem: u64,
}

impl Splash {
    pub fn auto() -> Result<Self> {
        let path = std::env::var("PATH").unwrap_or_else(|_| "unknown".to_string());

        let synth_ver = version();

        let synth_ref = META_SHORTNAME.to_string();
        let synth_rev = META_OID.to_string();
        let os = META_OS.to_string();
        let arch = META_ARCH.to_string();

        let system = System::new_all();
        let mem = system.get_total_memory();

        Ok(Self {
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
            arch = self.arch.bold(),
            os = self.os.bold(),
            mem = self.mem,
            cpu = num_cpus::get()
        )?;
        Ok(())
    }
}

#[cfg(feature = "api")]
pub async fn serve_daemon(args: ServeArgs) -> Result<()> {
    let daemon = std::sync::Arc::new(Daemon::new(args.data_directory.0)?);

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

#[allow(unused_variables)]
pub fn init_logger(args: &Args) {
    let mut loggers = Vec::<Box<dyn log::Log>>::new();

    // Env logger
    let env_logger = env_logger::Builder::from_default_env().build();
    loggers.push(Box::new(env_logger));
    #[cfg(feature = "api")]
    if let Args::Serve(ServeArgs {
        zenduty: Some(api_key),
        ..
    }) = args
    {
        let zen_logger = Box::new(crate::rlog::target::TargetLogger::new(
            "remote".to_string(),
            crate::rlog::zenduty::ZenDuty::new(api_key.clone()),
        ));
        loggers.push(zen_logger);
    }

    CompositeLogger::init(loggers)
}
