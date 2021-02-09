use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use git2::Repository;

#[macro_use]
extern crate quote;

fn git_data(repo_src: PathBuf) -> Result<(String, String), Box<dyn std::error::Error>> {
    let repo = Repository::open(repo_src)?;
    let head = repo.head()?;
    let oid = head.target().expect("a valid oid").to_string();
    let shortname = head.shorthand().expect("a valid shortname").to_string();
    Ok((oid, shortname))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repo_src = PathBuf::from(&env::var("SYNTH_SRC").unwrap_or("./.".to_string()));
    let (oid, shortname) = git_data(repo_src).unwrap_or(("unknown".to_string(), "unknown".to_string()));
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let mut f = File::create(format!("{}/meta.rs", env::var("OUT_DIR").unwrap()))?;
    write!(
        &mut f,
        "{}",
        quote! {
        const META_OID: &'static str = #oid;
        const META_SHORTNAME: &'static str = #shortname;
        const META_OS: &'static str = #os;
        const META_ARCH: &'static str = #arch;
        }
    )?;
    Ok(())
}
