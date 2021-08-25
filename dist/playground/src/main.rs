#![feature(try_blocks)]

pub mod prelude;
use prelude::*;

#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;

mod app;
use app::serve;

pub use structopt::StructOpt;

#[derive(StructOpt)]
enum Args {
    Serve(ServeCmd),
}

#[derive(StructOpt, Debug)]
pub struct ServeCmd {
    #[structopt(long, short, default_value = "127.0.0.1")]
    addr: IpAddr,
    #[structopt(long, short, default_value = "8182")]
    port: u16,
    #[structopt(long, short, default_value = "/playground")]
    mount: String,
    #[structopt(long, default_value = "1024")]
    max_size: usize,
    #[structopt(long, default_value = "POST,PUT,OPTIONS,GET")]
    allow_methods: String,
    #[structopt(long, default_value = "*")]
    allow_origin: String,
}

#[async_std::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    match Args::from_args() {
        Args::Serve(serve_cmd) => {
            serve(serve_cmd).await?;
        }
    }
    Ok(())
}
