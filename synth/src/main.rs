use anyhow::Result;
use structopt::StructOpt;
use synth::cli::Args;
use synth::cli::Cli;



#[async_std::main]
async fn main() -> Result<()> {

    let args = Args::from_args();
    let cli = Cli::new()?;

    #[cfg(feature = "telemetry")]
    synth::cli::telemetry::with_telemetry(args, |args| cli.run(args)).await?;

    #[cfg(not(feature = "telemetry"))]
    cli.run(args).await?;

    // Result ignored as this should fail silently
    let _ = synth::version::notify_new_version();

    Ok(())
}
