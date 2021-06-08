use anyhow::Result;
use log::debug;
use structopt::StructOpt;

use synth::{Args, cli, init_logger, META_OS, serve_daemon, Splash, version};

#[async_std::main]
async fn main() -> Result<()> {
    let args = Args::from_args();

    init_logger(&args);

    let splash = Splash::auto()?;
    debug!("{}", splash);

    match args {
        Args::Serve(sa) => serve_daemon(sa).await,
        Args::Cli(cli_args) => {
            cli::Cli::new(cli_args, version(), META_OS.to_string())?
                .run()
                .await
        }
    }
}
