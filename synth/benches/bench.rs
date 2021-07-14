// we can ignore irrefutable patterns here, because we might run this with or without a feature
#![allow(irrefutable_let_patterns)]
use log::debug;

use synth::{cli, init_logger, version, Args, Splash, META_OS};

fn bench_init() {
    async_std::task::block_on(async {
        let args = Args::Cli(cli::CliArgs::Init { init_path: None });

        init_logger(&args);

        let splash = Splash::auto().unwrap();
        debug!("{}", splash);

        if let Args::Cli(cli_args) = args {
            let _ = cli::Cli::new(cli_args, version(), META_OS.to_string())
                .unwrap()
                .run()
                .await;
        }
    });
}

fn bench_generate_1_to_stdout() {
    bench_generate_n_to_stdout(1);
}

fn bench_generate_100_to_stdout() {
    bench_generate_n_to_stdout(100);
}

fn bench_generate_10000_to_stdout() {
    bench_generate_n_to_stdout(10000);
}

fn bench_generate_n_to_stdout(size: usize) {
    async_std::task::block_on(async {
        let args = Args::Cli(cli::CliArgs::Generate {
            namespace: std::path::PathBuf::from("testing_harness/postgres/hospital_master"),
            collection: None,
            size,
            to: None,
            seed: Some(0),
            random: false,
        });

        init_logger(&args);

        let splash = Splash::auto().unwrap();
        debug!("{}", splash);

        if let Args::Cli(cli_args) = args {
            cli::Cli::new(cli_args, version(), META_OS.to_string())
                .unwrap()
                .run()
                .await
                .unwrap()
        }
    });
}

iai::main!(
    bench_init,
    bench_generate_1_to_stdout,
    bench_generate_100_to_stdout,
    bench_generate_10000_to_stdout,
);
