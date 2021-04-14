use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use git2::Repository;

use quote::quote;

fn git_data(repo_src: PathBuf) -> Result<(String, String), Box<dyn std::error::Error>> {
    let repo = Repository::open(repo_src)?;
    let head = repo.head()?;
    let oid = head.target().expect("a valid oid").to_string();
    let shortname = head.shorthand().expect("a valid shortname").to_string();
    Ok((oid, shortname))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repo_src = env::var("SYNTH_SRC").map_or_else(
        |_| {
            let mut p = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
            p.pop();
            p
        },
        PathBuf::from,
    );
    let (oid, shortname) =
        git_data(repo_src).unwrap_or(("unknown".to_string(), "unknown".to_string()));
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("meta.rs");

    let mut f = File::create(path)?;
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
