use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::path::PathBuf;
use std::future::Future;
use std::error::Error;
use uuid::Uuid;

use crate::utils::META_OS;
use crate::version::version;
use crate::cli::config;

use super::{Args, TelemetryCommand};

const API_KEY: &str = "L-AQtrFVtZGL_PjK2FbFLBR3oXNtfv8OrCD8ObyeBQo";
const EVENT_NAME: &str = "synth-command";

pub(crate) fn enable() -> Result<()> {
    // Initialise the `uuid` if it hasn't been initialised yet.
    let _ = get_or_initialise_uuid();
    Ok(config::set_telemetry_enabled(true))
}

pub(crate) fn disable() -> Result<()> {
    Ok(config::set_telemetry_enabled(false))
}

pub(crate) fn is_enabled() -> bool {
    config::get_telemetry_enabled().unwrap_or(false)
}

fn get_or_initialise_uuid() -> String {
    if config::get_uuid().is_none() {
        config::set_uuid(Uuid::new_v4().to_hyphenated().to_string());
    }
    config::get_uuid().expect("is ok here as was set earlier")
}

pub async fn with_telemetry<F, Fut, T, E>(args: Args, func: F) -> Result<T, E>
where
    F: FnOnce(Args) -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: AsRef<dyn Error + 'static>
{
    let client = TelemetryClient::new();

    let command_name = match &args {
        Args::Init { .. } => "init",
        Args::Generate { .. } => "generate",
        Args::Import { .. } => "import",
        Args::Telemetry(TelemetryCommand::Enable) => "telemetry::enable",
        Args::Telemetry(TelemetryCommand::Disable) => "telemetry::disable",
        Args::Telemetry(TelemetryCommand::Status) => "telemetry::status"
    };

    func(args)
        .await
        .and_then(|success| client.success(command_name, success))
        .or_else(|err| client.failed(command_name, err))
}

enum CommandResult {
    Success,
    Failed,
}

impl ToString for CommandResult {
    fn to_string(&self) -> String {
        match self {
            CommandResult::Success => "success".to_string(),
            CommandResult::Failed => "failed".to_string(),
        }
    }
}

pub(crate) struct TelemetryClient {
    ph_client: posthog_rs::Client,
    uuid: String,
    synth_version: String,
    os: String,
    enabled: bool,
}

impl TelemetryClient {
    fn new() -> Self {
        let synth_version = version();
        let os = META_OS.to_string();

        Self {
            ph_client: posthog_rs::client(API_KEY),
            uuid: get_or_initialise_uuid(),
            synth_version,
            os,
            enabled: is_enabled(),
        }
    }

    pub fn success<T, E>(&self, command_name: &str, output: T) -> Result<T, E> {
        self.send(command_name, CommandResult::Success)
            .or_else(|err| {
                info!("failed to push ok of command: {}", err);
                Ok(())
            })?;
        Ok(output)
    }

    pub fn failed<T, E>(&self, command_name: &str, error: E) -> Result<T, E>
    where
        E: AsRef<dyn Error + 'static>
    {
        self.send(command_name, CommandResult::Failed)
            .or_else(|err| {
                info!("failed to push err of command: {}", err);
                Ok(())
            })?;
        Err(error)
    }

    fn send(&self, command_name: &str, res: CommandResult) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let mut prop_map = HashMap::new();

        prop_map.insert("command".to_string(), command_name.to_string());
        prop_map.insert("success".to_string(), res.to_string());
        prop_map.insert("version".to_string(), self.synth_version.clone());
        prop_map.insert("os".to_string(), self.os.clone());

        let props = posthog_rs::Properties {
            distinct_id: self.uuid.clone(),
            props: prop_map,
        };

        let event = posthog_rs::Event {
            event: EVENT_NAME.to_string(),
            properties: props,
            timestamp: None,
        };

        if let Err(err) = self.ph_client.capture(event) {
            debug!("Failed to send message to PostHog. Error: {:?}", err);
        }

        Ok(())
    }
}
