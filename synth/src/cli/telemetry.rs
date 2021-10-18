use anyhow::{Context, Result};
use backtrace::Backtrace;
use colored::Colorize;
use lazy_static::lazy_static;

use std::collections::HashMap;
use std::error::Error;
use std::future::Future;
use std::io::{self, BufRead, Read, Write};
use std::panic;
use uuid::Uuid;

use crate::cli::config;
use crate::utils::META_OS;
use crate::version::version;

use super::{Args, TelemetryCommand};

const API_KEY: &str = "L-AQtrFVtZGL_PjK2FbFLBR3oXNtfv8OrCD8ObyeBQo";
const EVENT_NAME: &str = "synth-command";

lazy_static! {
    static ref TELEMETRY_CLIENT: TelemetryClient = TelemetryClient::new();
}

fn send_panic_report(synth_command: &str, telemetry_client: &TelemetryClient) {
    // This exists only to return early instead of panicking inside this panic hook
    let send_panic_report_impl = || -> Result<()> {
        let (stdin, stderr) = (io::stdin(), io::stderr());
        let (mut stdin, mut stderr) = (stdin.lock(), stderr.lock());
        let mut username = None;
        let mut email = None;

        eprintln!("\n{}", "Synth encountered a fatal error. Because telemetry is enabled, an anonymous \
            bug report will be sent to the Synth developers so that we may investigate the problem.".red());

        // Only interact with the user if there is an actual user running Synth; if the program's
        // stderr was redirected or this is running as part of CI, skip the interaction
        if console::user_attended_stderr() {
            eprint!("\nWould you also like to send your name and e-mail? One of our developers may enter \
                in contact to obtain further information if deemed necessary.\nAnswer (y/n): ");
            stderr.flush()?;

            // Newline character is also stored
            let mut answer = [0; 2];
            stdin
                .read_exact(&mut answer)
                .context("Couldn't read answer.")?;

            if answer[0] == b'y' {
                eprint!("Name: ");
                stderr.flush()?;

                let mut n = String::new();
                stdin.read_line(&mut n).context("Couldn't read name.")?;
                username = Some(n.trim().to_owned());

                eprint!("E-mail: ");
                stderr.flush()?;

                let mut e = String::new();
                stdin.read_line(&mut e).context("Couldn't read e-mail.")?;
                email = Some(e.trim().to_owned());
            }
        }

        let backtrace = Backtrace::new();
        let panic_report = PanicReport::new(username, email, synth_command.to_owned(), backtrace);

        telemetry_client.send_panic_report(panic_report)?;

        Ok(())
    };

    match send_panic_report_impl() {
        Ok(()) => eprintln!("Bug report sent with success. Thanks for your patience!"),
        Err(e) => eprintln!("Error sending bug report: {}", e),
    }
}

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
    E: AsRef<dyn Error + 'static>,
{
    if !is_enabled() {
        return func(args).await;
    }

    let default_panic_hook = panic::take_hook();
    let synth_command = serde_json::to_string(&args).unwrap();

    // Register a custom panic hook that will run the default hook and send a bug report to PostHog
    panic::set_hook(Box::new(move |panic_info| {
        default_panic_hook(panic_info);
        send_panic_report(&synth_command, &TELEMETRY_CLIENT);
    }));

    let command_name = match &args {
        Args::Init { .. } => "init",
        Args::Generate { .. } => "generate",
        Args::Import { .. } => "import",
        Args::Telemetry(TelemetryCommand::Enable) => "telemetry::enable",
        Args::Telemetry(TelemetryCommand::Disable) => "telemetry::disable",
        Args::Telemetry(TelemetryCommand::Status) => "telemetry::status",
        Args::Version => "version",
    };

    func(args)
        .await
        .and_then(|success| TELEMETRY_CLIENT.success(command_name, success))
        .or_else(|err| TELEMETRY_CLIENT.failed(command_name, err))
}

struct PanicReport {
    username: Option<String>,
    email: Option<String>,
    synth_command: String,
    backtrace: Backtrace,
}

impl PanicReport {
    fn new(
        username: Option<String>,
        email: Option<String>,
        synth_command: String,
        backtrace: Backtrace,
    ) -> Self {
        Self {
            username,
            email,
            synth_command,
            backtrace,
        }
    }
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
        }
    }

    fn default_telemetry_properties(&self) -> HashMap<String, String> {
        let mut prop_map = HashMap::new();
        prop_map.insert("version".to_string(), self.synth_version.clone());
        prop_map.insert("os".to_string(), self.os.clone());

        prop_map
    }

    pub fn success<T, E>(&self, command_name: &str, output: T) -> Result<T, E> {
        let mut prop_map = self.default_telemetry_properties();
        prop_map.insert("command".to_string(), command_name.to_string());
        prop_map.insert("success".to_string(), CommandResult::Success.to_string());

        self.send(EVENT_NAME.to_string(), prop_map).or_else(|err| {
            info!("failed to push ok of command: {}", err);
            Ok(())
        })?;
        Ok(output)
    }

    pub fn failed<T, E>(&self, command_name: &str, error: E) -> Result<T, E>
    where
        E: AsRef<dyn Error + 'static>,
    {
        let mut prop_map = self.default_telemetry_properties();
        prop_map.insert("command".to_string(), command_name.to_string());
        prop_map.insert("success".to_string(), CommandResult::Failed.to_string());

        self.send(EVENT_NAME.to_string(), prop_map).or_else(|err| {
            info!("failed to push err of command: {}", err);
            Ok(())
        })?;
        Err(error)
    }

    fn send_panic_report(&self, mut panic_report: PanicReport) -> Result<()> {
        panic_report.backtrace.resolve();

        let mut prop_map = self.default_telemetry_properties();
        prop_map.insert(
            "username".to_string(),
            panic_report.username.unwrap_or_default(),
        );
        prop_map.insert("email".to_string(), panic_report.email.unwrap_or_default());
        prop_map.insert("synth_command".to_string(), panic_report.synth_command);
        prop_map.insert(
            "backtrace".to_string(),
            format!("{:?}", panic_report.backtrace),
        );

        self.send(String::from("synth-panic-report"), prop_map)
    }

    fn send(&self, event: String, prop_map: HashMap<String, String>) -> Result<()> {
        let props = posthog_rs::Properties {
            distinct_id: self.uuid.clone(),
            props: prop_map,
        };

        let event = posthog_rs::Event {
            event,
            properties: props,
            timestamp: None,
        };

        if let Err(err) = self.ph_client.capture(event) {
            debug!("Failed to send message to PostHog. Error: {:?}", err);
            return Err(anyhow!("Failed to send message to PostHog."));
        }

        Ok(())
    }
}
