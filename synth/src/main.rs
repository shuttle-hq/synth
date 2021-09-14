use anyhow::Result;
use structopt::StructOpt;
use synth::cli::Args;
use synth::cli::Cli;

fn version() -> String {
    let current_version = synth::utils::version();
    let version_update_info = synth::utils::version_update_info()
        .unwrap_or_default()
        .unwrap_or_default();
    format!("{}\n{}", current_version, version_update_info)
}

fn setup_args() -> Args {
    let version = version();

    let mut app = Args::clap();
    app = app.version(version.as_str());

    Args::from_clap(&app.get_matches())
}

#[async_std::main]
async fn main() -> Result<()> {

    let args = setup_args();
    let cli = Cli::new()?;

    #[cfg(feature = "telemetry")]
        synth::cli::telemetry::with_telemetry(args, |args| cli.run(args)).await?;

    #[cfg(not(feature = "telemetry"))]
        cli.run(args).await?;

    // Result ignored as this should fail silently
    let _ = synth::utils::notify_new_version();

    Ok(())
}
