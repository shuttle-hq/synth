use anyhow::{Context, Error, Result};
use backtrace::Backtrace;
use colored::Colorize;
use lazy_static::lazy_static;

use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::future::Future;
use std::hash::{Hash, Hasher};
use std::io::{self, BufRead, Read, Write};
use std::panic;
use std::path::PathBuf;
use std::rc::Rc;
use uuid::Uuid;

use crate::cli::config;
use crate::cli::export::{ExportParams, ExportStrategy};
use crate::sampler::SamplerOutput;
use crate::utils::META_OS;
use crate::version::version;

use synth_core::{
    compile::{Address, CompilerState, FromLink, Source},
    Compile, Compiler, Content, Graph, Name, Namespace,
};

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

#[derive(Clone)]
#[cfg_attr(test, derive(Debug, Default, PartialEq, Eq))]
pub struct TelemetryContext {
    generators: Vec<String>,
    num_collections: Option<usize>,
    num_fields: Option<usize>,
    namespace_name_sha: Option<u64>,
    namespace_sha: Option<u64>,
    bytes: Option<usize>,
}

impl TelemetryContext {
    pub(super) fn new() -> Self {
        TelemetryContext {
            generators: Vec::new(),
            num_collections: None,
            num_fields: None,
            namespace_name_sha: None,
            namespace_sha: None,
            bytes: None,
        }
    }

    fn add_generator(&mut self, name: String) {
        self.generators.push(name);
    }

    pub(super) fn set_num_collections(&mut self, num: usize) {
        self.num_collections = Some(num);
    }

    fn inc_num_fields(&mut self) {
        self.num_fields = if let Some(num) = self.num_fields {
            Some(num + 1)
        } else {
            Some(1)
        }
    }
}

struct TelemetryCrawler<'t, 'a> {
    state: &'t mut CompilerState<'a, Graph>,
    position: Address,
    context: Rc<RefCell<TelemetryContext>>,
}

impl<'t, 'a: 't> TelemetryCrawler<'t, 'a> {
    fn as_at(&mut self, field: &str, content: &'a Content) -> TelemetryCrawler<'_, 'a> {
        let position = self.position.clone().into_at(field);
        TelemetryCrawler {
            state: self.state.entry(field).or_init(content),
            position,
            context: Rc::clone(&self.context),
        }
    }

    fn compile(self) -> Result<()> {
        match self.state.source() {
            Source::Namespace(namespace) => namespace.compile(self)?,
            Source::Content(content) => content.compile(self)?,
        };
        Ok(())
    }
}

impl<'t, 'a: 't> Compiler<'a> for TelemetryCrawler<'t, 'a> {
    fn build(&mut self, field: &str, content: &'a Content) -> Result<Graph> {
        self.context
            .borrow_mut()
            .add_generator(format!("{}", content));
        self.context.borrow_mut().inc_num_fields();

        if let Err(err) = self.as_at(field, content).compile() {
            warn!(
                "could not crawl into field `{}` at `{}`",
                field, self.position
            );
            return Err(err);
        }
        Ok(Graph::dummy())
    }

    fn get<S: Into<Address>>(&mut self, _target: S) -> Result<Graph> {
        Ok(Graph::dummy())
    }
}

pub(super) struct TelemetryExportStrategy {
    exporter: Box<dyn ExportStrategy>,
    telemetry_context: Rc<RefCell<TelemetryContext>>,
}

impl TelemetryExportStrategy {
    pub fn new(strategy: Box<dyn ExportStrategy>, context: Rc<RefCell<TelemetryContext>>) -> Self {
        TelemetryExportStrategy {
            exporter: strategy,
            telemetry_context: context,
        }
    }

    pub(super) fn fill_telemetry_pre(
        context: Rc<RefCell<TelemetryContext>>,
        namespace: &Namespace,
        collection: Option<Name>,
        ns_path: PathBuf,
    ) -> Result<()> {
        let crawler = TelemetryCrawler {
            state: &mut CompilerState::namespace(namespace),
            position: Address::new_root(),
            context: Rc::clone(&context),
        };

        if let Some(name) = collection {
            if let Some(content) = namespace.collections.get(&name) {
                content.compile(crawler)?;
                context.borrow_mut().num_collections = Some(1);

                // For length and content
                if let Some(ref mut n) = context.borrow_mut().num_fields {
                    *n -= 2;
                }
            }
        } else {
            namespace.compile(crawler)?;
            let num_col = namespace.collections.len();
            context.borrow_mut().num_collections = Some(num_col);

            // Namespace, length and content for each collection
            if let Some(ref mut n) = context.borrow_mut().num_fields {
                *n -= 3 * num_col;
            }
        }

        let mut context_mut = context.borrow_mut();

        let mut hasher = DefaultHasher::new();
        ns_path.hash(&mut hasher);
        context_mut.namespace_name_sha = Some(hasher.finish());

        hasher = DefaultHasher::new();
        namespace.hash(&mut hasher);
        context_mut.namespace_sha = Some(hasher.finish());

        Ok(())
    }

    fn fill_telemetry_post(&self, output: SamplerOutput) -> Result<()> {
        let j = output.into_json();
        let s = serde_json::to_string(&j)?;

        self.telemetry_context.borrow_mut().bytes = Some(s.len());

        Ok(())
    }
}

impl ExportStrategy for TelemetryExportStrategy {
    fn export(&self, params: ExportParams) -> Result<SamplerOutput> {
        Self::fill_telemetry_pre(
            Rc::clone(&self.telemetry_context),
            &params.namespace,
            params.collection_name.clone(),
            params.ns_path.clone(),
        )?;
        let output = self.exporter.export(params)?;

        self.fill_telemetry_post(output.clone())?;

        Ok(output)
    }
}

pub async fn with_telemetry<F, Fut, T, G>(
    args: Args,
    func: F,
    func_telemetry_context: G,
) -> Result<T>
where
    F: FnOnce(Args) -> Fut,
    Fut: Future<Output = Result<T>>,
    G: FnOnce() -> TelemetryContext,
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
        .and_then(|success| {
            TELEMETRY_CLIENT.success(command_name, success, func_telemetry_context())
        })
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

    fn default_telemetry_event<S: Into<String>>(&self, event: S) -> Result<posthog_rs::Event> {
        let mut event = posthog_rs::Event::new(event.into(), self.uuid.clone());

        event.insert_prop("version", self.synth_version.clone())?;
        event.insert_prop("os", self.os.clone())?;

        Ok(event)
    }

    fn add_telemetry_context(
        event: &mut posthog_rs::Event,
        telemetry_context: TelemetryContext,
    ) -> Result<()> {
        if telemetry_context.generators.len() > 0 {
            event.insert_prop("generators", telemetry_context.generators)?;
        }

        if let Some(num_collections) = telemetry_context.num_collections {
            if num_collections > 0 {
                event.insert_prop("num_collections", num_collections)?;

                if let Some(num_fields) = telemetry_context.num_fields {
                    let avg = num_fields as f64 / num_collections as f64;
                    event.insert_prop("avg_num_fields_per_collection", avg)?;
                }
            }
        }

        if let Some(namespace_sha) = telemetry_context.namespace_sha {
            event.insert_prop("namespace_sha", namespace_sha)?;
        }

        if let Some(namespace_name_sha) = telemetry_context.namespace_name_sha {
            event.insert_prop("namespace_name_sha", namespace_name_sha)?;
        }

        if let Some(bytes) = telemetry_context.bytes {
            event.insert_prop("bytes", bytes)?;
        }

        Ok(())
    }

    pub fn success<T>(
        &self,
        command_name: &str,
        output: T,
        telemetry_context: TelemetryContext,
    ) -> Result<T> {
        let mut event = self.default_telemetry_event(EVENT_NAME)?;
        event.insert_prop("command", command_name)?;
        event.insert_prop("success", CommandResult::Success.to_string())?;

        TelemetryClient::add_telemetry_context(&mut event, telemetry_context)?;

        self.send(event).or_else::<Error, _>(|err| {
            info!("failed to push ok of command: {}", err);
            Ok(())
        })?;
        Ok(output)
    }

    pub fn failed<T>(&self, command_name: &str, error: Error) -> Result<T> {
        let mut event = self.default_telemetry_event(EVENT_NAME)?;

        event.insert_prop("command", command_name)?;
        event.insert_prop("success", CommandResult::Failed.to_string())?;

        self.send(event).or_else::<Error, _>(|err| {
            info!("failed to push err of command: {}", err);
            Ok(())
        })?;

        Err(error)
    }

    fn send_panic_report(&self, mut panic_report: PanicReport) -> Result<()> {
        panic_report.backtrace.resolve();

        let mut event = self.default_telemetry_event("synth-panic-report")?;

        event.insert_prop("username", panic_report.username.unwrap_or_default())?;
        event.insert_prop("email", panic_report.email.unwrap_or_default())?;
        event.insert_prop("synth_command", panic_report.synth_command)?;
        event.insert_prop("backtrace", format!("{:?}", panic_report.backtrace))?;

        self.send(event)
    }

    fn send(&self, event: posthog_rs::Event) -> Result<()> {
        if let Err(err) = self.ph_client.capture(event) {
            debug!("Failed to send message to PostHog. Error: {:?}", err);
            return Err(anyhow!("Failed to send message to PostHog."));
        }

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::{
        ExportParams, ExportStrategy, Namespace, SamplerOutput, TelemetryClient, TelemetryContext,
        TelemetryExportStrategy,
    };
    use crate::sampler::Sampler;
    use anyhow::Result;
    use std::cell::RefCell;
    use std::convert::TryFrom;
    use std::path::PathBuf;
    use std::rc::Rc;

    macro_rules! schema {
        {
            $($inner:tt)*
        } => {
            serde_json::from_value::<synth_core::schema::Content>(serde_json::json!($($inner)*))
                .expect("could not deserialize value into a schema")
        }
    }

    pub struct DummyExportStrategy {}

    impl ExportStrategy for DummyExportStrategy {
        fn export(&self, params: ExportParams) -> Result<SamplerOutput> {
            let generator = Sampler::try_from(&params.namespace)?;
            let output =
                generator.sample_seeded(params.collection_name, params.target, params.seed)?;

            Ok(output)
        }
    }

    #[test]
    fn telemetry_export_strategy() {
        let mut schema: Namespace = schema!({
            "type": "object",
            "strings": {
                "type": "array",
                "length": 1,
                "content": {
                    "type": "object",
                    "username": {
                        "type": "string",
                        "faker": {
                            "generator": "username"
                        }
                    },
                    "credit_card": {
                        "type": "string",
                        "faker": {
                            "generator": "credit_card"
                        }
                    },
                    "email": {
                        "type": "string",
                        "faker": {
                            "generator": "safe_email"
                        }
                    }
                }
            },
            "extra": {
                "type": "array",
                "length": 1,
                "content": {
                    "type": "object",
                    "str": {
                        "type": "string",
                        "pattern": "extra collection test"
                    }
                }
            }
        })
        .into_namespace()
        .unwrap();

        let context = Rc::new(RefCell::new(TelemetryContext::new()));
        let export_strategy =
            TelemetryExportStrategy::new(Box::new(DummyExportStrategy {}), Rc::clone(&context));

        export_strategy
            .export(ExportParams {
                namespace: schema,
                collection_name: None,
                target: 1,
                seed: 500,
                ns_path: PathBuf::from("/dummy/path"),
            })
            .unwrap();

        assert_eq!(
            context.take(),
            TelemetryContext {
                generators: vec!(
                    "array".to_string(),
                    "number::U64::Constant".to_string(),
                    "object".to_string(),
                    "string::pattern".to_string(),
                    "array".to_string(),
                    "number::U64::Constant".to_string(),
                    "object".to_string(),
                    "string::credit_card".to_string(),
                    "string::safe_email".to_string(),
                    "string::username".to_string()
                ),
                num_collections: Some(2),
                num_fields: Some(4),
                namespace_name_sha: Some(15302706232490290083),
                namespace_sha: Some(10334654735128021143),
                bytes: Some(138),
            },
            "string fakers should be correct"
        );

        schema = schema!({
            "type": "object",
            "integers": {
                "type": "array",
                "length": 1,
                "content": {
                    "type": "object",
                    "int_64": {
                        "type": "number",
                        "subtype": "i64",
                        "range": {
                            "low": -5
                        }
                    },
                    "uint_64": {
                        "type": "number",
                        "subtype": "u64",
                        "range": {
                            "high": 5
                        }
                    },
                    "f_64": {
                        "type": "number",
                        "subtype": "f64",
                        "range": {
                            "high": 3.2
                        }
                    },
                    "int_32": {
                        "type": "number",
                        "subtype": "i32",
                        "range": {
                            "low": -13
                        }
                    },
                    "uint_32": {
                        "type": "number",
                        "subtype": "u32",
                        "range": {
                            "high": 23
                        }
                    },
                    "f_32": {
                        "type": "number",
                        "subtype": "f32",
                        "range": {
                            "high": 21.2
                        }
                    }
                }
            }
        })
        .into_namespace()
        .unwrap();

        export_strategy
            .export(ExportParams {
                namespace: schema,
                collection_name: None,
                target: 1,
                seed: 500,
                ns_path: PathBuf::from("/dummy/path"),
            })
            .unwrap();

        assert_eq!(
            context.take(),
            TelemetryContext {
                generators: vec!(
                    "array".to_string(),
                    "number::U64::Constant".to_string(),
                    "object".to_string(),
                    "number::F32::Range".to_string(),
                    "number::F64::Range".to_string(),
                    "number::I32::Range".to_string(),
                    "number::I64::Range".to_string(),
                    "number::U32::Range".to_string(),
                    "number::U64::Range".to_string(),
                ),
                num_collections: Some(1),
                num_fields: Some(6),
                namespace_name_sha: Some(15302706232490290083),
                namespace_sha: Some(7413555395156765757),
                bytes: Some(140),
            },
            "int ranges should be correct"
        );

        schema = schema!({
            "type": "object",
            "generators": {
                "type": "array",
                "length": 1,
                "content": {
                    "type": "object",
                    "constant": {
                        "type": "bool",
                        "constant": true
                    },
                    "frequency": {
                        "type": "bool",
                        "frequency": 0.65
                    },
                    "incrementing": {
                        "type": "series",
                        "incrementing": {
                            "start": "2021-02-01 09:00:00",
                            "increment": "1m"
                        }
                    },
                    "poisson": {
                        "type": "series",
                        "poisson": {
                            "start": "2021-02-01 09:00:00",
                            "rate": "1m"
                        }
                    },
                    "cyclic": {
                        "type": "series",
                        "cyclical": {
                            "start": "2021-02-01 00:00:00",
                            "period": "1d",
                            "min_rate": "10m",
                            "max_rate": "30s"
                        }
                    }
                }
            }
        })
        .into_namespace()
        .unwrap();

        export_strategy
            .export(ExportParams {
                namespace: schema,
                collection_name: None,
                target: 1,
                seed: 500,
                ns_path: PathBuf::from("/dummy/path"),
            })
            .unwrap();

        assert_eq!(
            context.take(),
            TelemetryContext {
                generators: vec!(
                    "array".to_string(),
                    "number::U64::Constant".to_string(),
                    "object".to_string(),
                    "bool::constant".to_string(),
                    "series::cyclical".to_string(),
                    "bool::frequency".to_string(),
                    "series::incrementing".to_string(),
                    "series::poisson".to_string(),
                ),
                num_collections: Some(1),
                num_fields: Some(5),
                namespace_name_sha: Some(15302706232490290083),
                namespace_sha: Some(1231538834371280080),
                bytes: Some(151),
            },
            "other generators should be correct"
        );

        schema = schema!({
            "type": "object",
            "collection-1": {
                "type": "array",
                "length": 1,
                "content": {
                    "type": "object",
                    "str": {
                        "type": "string",
                        "faker": {
                            "generator": "username"
                        }
                    }
                }
            },
            "collection-2": {
                "type": "array",
                "length": 1,
                "content": {
                    "type": "object",
                    "str": {
                        "type": "string",
                        "faker": {
                            "generator": "credit_card"
                        }
                    }
                }
            },
            "collection-3": {
                "type": "array",
                "length": 1,
                "content": {
                    "type": "object",
                    "str": {
                        "type": "string",
                        "faker": {
                            "generator": "safe_email"
                        }
                    }
                }
            }
        })
        .into_namespace()
        .unwrap();

        export_strategy
            .export(ExportParams {
                namespace: schema,
                collection_name: "collection-2".parse().ok(),
                target: 1,
                seed: 500,
                ns_path: PathBuf::from("/dummy/namespace"),
            })
            .unwrap();

        assert_eq!(
            context.take(),
            TelemetryContext {
                generators: vec!(
                    "number::U64::Constant".to_string(),
                    "object".to_string(),
                    "string::credit_card".to_string()
                ),
                num_collections: Some(1),
                num_fields: Some(1),
                namespace_name_sha: Some(6868949361385231259),
                namespace_sha: Some(5224534078999940403),
                bytes: Some(27),
            },
            "should only have stats for collection 2"
        );
    }

    #[test]
    fn add_telemetry_context() {
        let mut event = posthog_rs::Event::new("dummy", "id");
        let mut context = TelemetryContext::new();

        TelemetryClient::add_telemetry_context(&mut event, context.clone()).unwrap();
        assert_eq!(
            serde_json::to_string_pretty(&event).unwrap(),
            r#"{
  "event": "dummy",
  "properties": {
    "distinct_id": "id",
    "props": {}
  },
  "timestamp": null
}"#,
            "empty fields should not be added"
        );

        context.generators.push("string::name".to_string());
        context.generators.push("string::credit_card".to_string());
        TelemetryClient::add_telemetry_context(&mut event, context.clone()).unwrap();
        assert_eq!(
            serde_json::to_string_pretty(&event).unwrap(),
            r#"{
  "event": "dummy",
  "properties": {
    "distinct_id": "id",
    "props": {
      "generators": [
        "string::name",
        "string::credit_card"
      ]
    }
  },
  "timestamp": null
}"#,
            "generators should be separated by commas"
        );
        context.generators = Vec::new();

        event = posthog_rs::Event::new("dummy", "id");
        context.num_collections = Some(6);
        TelemetryClient::add_telemetry_context(&mut event, context.clone()).unwrap();
        assert_eq!(
            serde_json::to_string_pretty(&event).unwrap(),
            r#"{
  "event": "dummy",
  "properties": {
    "distinct_id": "id",
    "props": {
      "num_collections": 6
    }
  },
  "timestamp": null
}"#,
            "include num_collections"
        );

        event = posthog_rs::Event::new("dummy", "id");
        context.num_fields = Some(9);
        TelemetryClient::add_telemetry_context(&mut event, context.clone()).unwrap();
        assert_eq!(
            serde_json::to_string_pretty(&event).unwrap(),
            r#"{
  "event": "dummy",
  "properties": {
    "distinct_id": "id",
    "props": {
      "num_collections": 6,
      "avg_num_fields_per_collection": 1.5
    }
  },
  "timestamp": null
}"#,
            "include avg_fields_per_collection"
        );
        context.num_collections = None;
        context.num_fields = None;

        event = posthog_rs::Event::new("dummy", "id");
        context.num_fields = Some(9);
        context.namespace_sha = Some(50238);
        TelemetryClient::add_telemetry_context(&mut event, context.clone()).unwrap();
        assert_eq!(
            serde_json::to_string_pretty(&event).unwrap(),
            r#"{
  "event": "dummy",
  "properties": {
    "distinct_id": "id",
    "props": {
      "namespace_sha": 50238
    }
  },
  "timestamp": null
}"#,
            "include namespace_sha"
        );
        context.namespace_sha = None;

        event = posthog_rs::Event::new("dummy", "id");
        context.namespace_name_sha = Some(54321);
        TelemetryClient::add_telemetry_context(&mut event, context.clone()).unwrap();
        assert_eq!(
            serde_json::to_string_pretty(&event).unwrap(),
            r#"{
  "event": "dummy",
  "properties": {
    "distinct_id": "id",
    "props": {
      "namespace_name_sha": 54321
    }
  },
  "timestamp": null
}"#,
            "include namespace_name_sha"
        );
        context.namespace_name_sha = None;

        event = posthog_rs::Event::new("dummy", "id");
        context.bytes = Some(1024);
        TelemetryClient::add_telemetry_context(&mut event, context.clone()).unwrap();
        assert_eq!(
            serde_json::to_string_pretty(&event).unwrap(),
            r#"{
  "event": "dummy",
  "properties": {
    "distinct_id": "id",
    "props": {
      "bytes": 1024
    }
  },
  "timestamp": null
}"#,
            "include bytes"
        );
        context.bytes = None;

        // Edge cases
        event = posthog_rs::Event::new("dummy", "id");
        context.num_collections = Some(0);
        context.num_fields = Some(15);
        TelemetryClient::add_telemetry_context(&mut event, context.clone()).unwrap();
        assert_eq!(
            serde_json::to_string_pretty(&event).unwrap(),
            r#"{
  "event": "dummy",
  "properties": {
    "distinct_id": "id",
    "props": {}
  },
  "timestamp": null
}"#,
            "don't include avg_fields_per_collection when collection is 0"
        );

        event = posthog_rs::Event::new("dummy", "id");
        context.num_collections = None;
        context.num_fields = Some(4);
        TelemetryClient::add_telemetry_context(&mut event, context.clone()).unwrap();
        assert_eq!(
            serde_json::to_string_pretty(&event).unwrap(),
            r#"{
  "event": "dummy",
  "properties": {
    "distinct_id": "id",
    "props": {}
  },
  "timestamp": null
}"#,
            "don't include avg_fields_per_collection when collection is None"
        );
    }
}
