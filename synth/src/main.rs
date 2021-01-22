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
use pyo3::Python;

#[macro_use]
mod error;

mod api;
pub use api::Api;

mod daemon;
use daemon::Daemon;

use crate::rlog::composite::CompositeLogger;
use crate::rlog::target::TargetLogger;
use crate::rlog::zenduty::ZenDuty;

mod rlog;

pub mod index;
pub use index::Index;

pub mod store;

include!(concat!(env!("OUT_DIR"), "/meta.rs"));

struct DataDirectoryPath(PathBuf);

impl Default for DataDirectoryPath {
    fn default() -> Self {
        let home = env::var("HOME").unwrap_or("/".to_string());
        let default_path = if home == "/" {
            env::current_dir()
                .map(|pwd| pwd.to_str().unwrap().to_string())
                .unwrap_or("/".to_string())
        } else {
            shellexpand::tilde("~/.local/share/synthd").into_owned()
        };
        Self(default_path.into())
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
#[structopt(name = "synthd", about = "synthetic data engine")]
pub struct Args {
    #[structopt(short, long, default_value = "0.0.0.0:8182")]
    bind: SocketAddr,
    #[structopt(short, long, default_value)]
    data_directory: DataDirectoryPath,
    #[structopt(long)]
    zenduty: Option<String>,
}

struct Splash {
    python_ver: String,
    synthd_ver: String,
    synthd_ref: String,
    synthd_rev: String,
    os: String,
    arch: String,
    mem: u64,
}

impl Splash {
    fn auto() -> Result<Self> {
        #[cfg(feature = "python")]
        let python_ver = {
            let gil = Python::acquire_gil();
            let py = gil.python();
            let sys = py.import("sys")?;
            sys.get("version")?
                .extract()
                .map(|ver_str: String| ver_str.replace("\n", ""))
        }?;

        #[cfg(not(feature = "python"))]
        let python_ver = { "disabled".bold().red().to_string() };

        let synthd_ver = env!("CARGO_PKG_VERSION").to_string();

        let synthd_ref = META_SHORTNAME.to_string();
        let synthd_rev = META_OID.to_string();
        let os = META_OS.to_string();
        let arch = META_ARCH.to_string();

        let system = System::new_all();
        let mem = system.get_total_memory();

        Ok(Self {
            python_ver,
            synthd_ver,
            synthd_ref,
            synthd_rev,
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

version = {synthd_ver}
ref     = {synthd_ref}
rev     = {synthd_rev}
python  = {python_ver}
target  = {os}
arch    = {arch}
threads = {cpu}
mem     = {mem}
",
            synthd_ver = self.synthd_ver.blue().bold(),
            synthd_ref = self.synthd_ref.bold(),
            synthd_rev = self.synthd_rev.bold(),
            python_ver = self.python_ver.bold(),
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

    init_remote_logger(&args);

    let splash = Splash::auto()?;
    debug!("{}", splash);

    let daemon = Arc::new(Daemon::new(args.data_directory.0)?);

    let server = Api::new_server(daemon)?;
    eprintln!(
        "{} is listening on {}",
        "synthd".bold(),
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

fn init_remote_logger(args: &Args) {
    let mut loggers = Vec::<Box<dyn log::Log>>::new();

    // Env logger
    let env_logger = env_logger::Builder::from_default_env().build();
    loggers.push(Box::new(env_logger));

    // Zenduty Logger
    if let Some(api_key) = &args.zenduty {
        let zen_logger = Box::new(TargetLogger::new(
            "remote".to_string(),
            ZenDuty::new(api_key.clone()),
        ));
        loggers.push(zen_logger);
    }

    CompositeLogger::init(loggers)
}
