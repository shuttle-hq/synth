use anyhow::Result;
use std::thread;
use std::thread::JoinHandle;
use structopt::StructOpt;
use synth::cli::Args;
use synth::cli::Cli;

#[async_std::main]
async fn main() -> Result<()> {
    let args = Args::from_args();
    let cli = Cli::new()?;

    // The `synth version` command already checks for new Synth versions. Therefore, don't spawn
    // another thread that will do virtually the same task.
    let notify_handle = match args {
        Args::Version => None,
        _ => Some(thread::spawn(synth::version::notify_new_version_message)),
    };

    #[cfg(feature = "telemetry")]
    synth::cli::telemetry::with_telemetry(
        args,
        |args| cli.run(args),
        || cli.get_telemetry_context(),
    )
    .await?;

    #[cfg(not(feature = "telemetry"))]
    cli.run(args).await?;

    if let Some(notify_handle) = notify_handle {
        print_notify(notify_handle);
    }

    Ok(())
}

fn print_notify(handle: JoinHandle<Result<Option<String>>>) {
    if let Ok(Ok(Some(notify_message))) = handle.join() {
        eprintln!("{}", notify_message);
    }
}
