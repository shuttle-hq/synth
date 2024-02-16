#[cfg(debug_assertions)]
pub mod splash {
    use anyhow::Result;
    use owo_colors::OwoColorize;
    use sysinfo::{System, SystemExt};

    use crate::version::version;

    pub struct Splash {
        synth_ver: String,
        path: String,
        os: String,
        arch: String,
        mem: u64,
    }

    impl Splash {
        pub fn auto() -> Result<Self> {
            let path = std::env::var("PATH").unwrap_or_else(|_| "unknown".to_string());

            let synth_ver = version();

            let os = std::env::consts::OS.to_string();
            let arch = std::env::consts::ARCH.to_string();

            let system = System::new_all();
            let mem = system.total_memory();

            Ok(Self {
                synth_ver,
                path,
                os,
                arch,
                mem,
            })
        }
    }

    impl std::fmt::Display for Splash {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "

version     = {synth_ver}
PATH        = {path}
target      = {os}
arch        = {arch}
threads     = {cpu}
mem         = {mem}
",
                synth_ver = self.synth_ver.blue().bold(),
                path = self.path.bold(),
                arch = self.arch.bold(),
                os = self.os.bold(),
                mem = self.mem,
                cpu = num_cpus::get()
            )?;
            Ok(())
        }
    }
}
