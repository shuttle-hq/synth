use crate::cli::config;
use anyhow::Result;
use chrono::Utc;
use reqwest::header::USER_AGENT;
use semver::Version;
use serde_json::map::Map;
use serde_json::value::Value;
use std::io::Write;
use std::time::Duration;

/// This is used when the user does `synth version`.
/// It will always display a new version if it exists.
pub fn print_version_message<W: Write>(mut writer: W) {
    let current_version = version();
    let version_update_info = version_update_info()
        .map(|(info, _)| info)
        .unwrap_or_default()
        .map(|info| format!("\n{}", info))
        .unwrap_or_default();
    writeln!(writer, "synth {}{}", current_version, version_update_info)
        .expect("failed to write version");
}

// This is used when the user runs any command (except for version)
// If a new version is seen, it is notified once and then stored in
// config.
pub fn notify_new_version_message() -> Result<Option<String>> {
    let now = Utc::now();

    // If we already cached the GET request and it hasn't expired, return successfully.
    if let Some(version_check_delay) = config::get_version_check_delay() {
        if version_check_delay > now {
            return Ok(None);
        }
    }

    let (version_info, latest_version) = version_update_info()?;
    let mut ret = None;
    config::set_version_check_delay(now + chrono::Duration::days(1));

    // if this is `Some`, our version is out of date.
    if let Some(version_info) = version_info {
        if !has_notified_for_version(latest_version) {
            ret = Some(version_info);
        }
    }
    Ok(ret)
}

/// Notify the user if there is a new version of Synth
/// Even though the error is not meant to be used, it
/// makes the implementation simpler instead of returning ().
pub fn version_update_info() -> Result<(Option<String>, Version)> {
    let current_version = version_semver();
    let latest_version = latest_version()?;
    Ok((
        version_update_info_inner(&current_version, &latest_version),
        latest_version,
    ))
}

pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn version_semver() -> Version {
    Version::parse(&version()).unwrap()
}

fn has_notified_for_version(version: Version) -> bool {
    // If no versions have been seen yet, default to empty map.
    let mut seen_versions = config::get_seen_versions().unwrap_or_default();
    let version_as_string = version.to_string();

    // If the set did not have this value present, true is returned.
    let has_notified = !seen_versions.insert(version_as_string);

    // save seen versions
    config::set_seen_versions(seen_versions);

    has_notified
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
        .ok_or_else(|| anyhow!("could not get the 'name' parameter"))?
        .as_str()
        .ok_or_else(|| anyhow!("was expecting name to be a string"))?;

    // At this point it looks like 'vX.Y.Z'. Here we're removing the `v`
    // Maybe we should use something that doesn't panic?
    Version::parse(&latest_version[1..])
        .map_err(|e| anyhow!("failed to parse latest version semver with error: {}", e))
}

fn version_update_info_inner(
    current_version: &Version,
    latest_version: &Version,
) -> Option<String> {
    if latest_version > current_version {
        let out_of_date = "\nYour version of synth is out of date.";
        let version_compare = format!(
            "The installed version is {} and the latest version is {}.",
            current_version, latest_version
        );
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
