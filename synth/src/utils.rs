use std::env;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use serde_json::{Map, Value};
use reqwest::header::USER_AGENT;
use std::time::Duration;
use semver::Version;

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

fn version_semver() -> Result<Version> {
    Version::parse(&version())
        .map_err(|e| anyhow!("failed to parse current version semver with error: {}", e))
}

fn has_notified_for_version(version: Version) -> bool {
    unimplemented!()
}

pub fn notify_new_version() -> Result<()> {
    let (version_info, latest_version) = version_update_info()?;
    // if this is `Some`, our version is out of date.
    if let Some(version_info) = version_info {
        if !has_notified_for_version(latest_version) {
            eprintln!("{}", version_info);
        }
    }
    Ok(())
}

/// Notify the user if there is a new version of Synth
/// Even though the error is not meant to be used, it
/// makes the implementation simpler instead of returning ().
pub fn version_update_info() -> Result<(Option<String>, Version)> {
    let current_version = crate::utils::version_semver()?;
    let latest_version = latest_version()?;
    Ok((version_update_info_inner(&current_version, &latest_version), latest_version))
}

fn latest_version() -> Result<Version> {
    let url = "https://api.github.com/repos/getsynth/synth/releases/latest";
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(url)
        .header(USER_AGENT, "hyper/0.14")
        .timeout(Duration::from_secs(2))
        .send()?;

    let release_info: Map<String, Value> = response.json()?;

    // We're assuming here that the GH API doesn't make breaking changes
    // otherwise these `get` and `as_str` operations are quite safe
    let latest_version = release_info
        .get("name")
        .ok_or(anyhow!("could not get the 'name' parameter"))?
        .as_str()
        .ok_or(anyhow!("was expecting name to be a string"))?;

    // At this point it looks like 'vX.Y.Z'. Here we're removing the `v`
    // Maybe we should use something that doesn't panic?
    Version::parse(&latest_version[1..])
        .map_err(|e| anyhow!("failed to parse latest version semver with error: {}", e))
}

fn version_update_info_inner(current_version: &Version, latest_version: &Version) -> Option<String> {
    if latest_version > current_version {
        let out_of_date = "\nYour version of synth is out of date.";
        let version_compare = format!("The installed version is {} and the latest version is {}.", current_version, latest_version);
        #[cfg(windows)]
        let install_advice = "You can update by downloading from: https://github.com/getsynth/synth/releases/latest/download/synth-windows-latest-x86_64.exe";
        #[cfg(not(windows))]
        let install_advice = "You can update synth by running: curl --proto '=https' --tlsv1.2 -sSL https://getsynth.com/install | sh -s -- --force";

        let formatted = format!("{}\n{}\n{}\n", out_of_date, version_compare, install_advice);
        Some(formatted)
    } else {
        None
    }
}


#[cfg(debug_assertions)]
pub mod splash {
    use anyhow::Result;
    use colored::Colorize;
    use sysinfo::{System, SystemExt};

    use super::*;

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
}
