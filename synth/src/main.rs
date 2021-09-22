use anyhow::Result;
use structopt::StructOpt;
use synth::cli::Args;
use synth::cli::Cli;
use std::thread;
use std::thread::JoinHandle;

#[async_std::main]
async fn main() -> Result<()> {
    let args = Args::from_args();
    let cli = Cli::new()?;

    let notify_handle = thread::spawn(synth::version::notify_new_version_message);

    #[cfg(feature = "telemetry")]
    synth::cli::telemetry::with_telemetry(args, |args| cli.run(args)).await?;

    #[cfg(not(feature = "telemetry"))]
    cli.run(args).await?;

    print_notify(notify_handle);

    Ok(())
}

fn print_notify(handle: JoinHandle<Result<Option<String>>>) {
    if let Ok(Ok(Some(notify_message))) = handle.join() {
        eprintln!("{}", notify_message);
    }
}
