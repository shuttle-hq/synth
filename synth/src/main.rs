use anyhow::Result;
use log::debug;
use structopt::StructOpt;

use synth::{Args, cli, init_logger, META_OS, Splash, version};

#[async_std::main]
async fn main() -> Result<()> {
    let args = Args::from_args();

    init_logger(&args);

    let splash = Splash::auto()?;
    debug!("{}", splash);

    match args {
        #[cfg(feature = "api")]
        Args::Serve(sa) => synth::serve_daemon(sa).await,
        Args::Cli(cli_args) => {
            cli::Cli::new(cli_args, version(), META_OS.to_string())?
                .run()
                .await
        }
    }
}
