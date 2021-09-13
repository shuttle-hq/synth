use anyhow::Result;
use structopt::StructOpt;

#[async_std::main]
async fn main() -> Result<()> {
    let args = synth::cli::Args::from_args();
    let cli = synth::cli::Cli::new()?;

    #[cfg(feature = "telemetry")]
    synth::cli::telemetry::with_telemetry(args, |args| cli.run(args)).await?;

    #[cfg(not(feature = "telemetry"))]
    cli.run(args).await?;

    // Result ignored as this should fail silently
    let _ = synth::utils::notify_new_version();

    Ok(())
}
