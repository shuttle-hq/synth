use anyhow::Result;
use structopt::StructOpt;

use synth::cli::{Args, Cli};

#[async_std::main]
async fn main() -> Result<()> {
    Cli::new(Args::from_args())?.run().await
}
