use std::env;
use std::fs;
use std::io::Result;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let pretrained_path = env::var_os("PRETRAINED")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new("train").join("dummy.tch"));
    let target_path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("pretrained.tch");
    eprintln!(
        "attempting to copy pretrained weights:\n\t<- {}\n\t-> {}",
        pretrained_path.to_str().unwrap(),
        target_path.to_str().unwrap()
    );
    fs::copy(&pretrained_path, &target_path)?;
    println!(
        "cargo:rustc-env=PRETRAINED={}",
        target_path.to_str().unwrap()
    );
    Ok(())
}
