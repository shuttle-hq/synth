use anyhow::Result;
use std::path::PathBuf;
use structopt::StructOpt;
use synth_core::schema::Namespace;

pub(crate) trait ImportStrategy {
    fn import(self) -> Result<Namespace>;
}

#[derive(StructOpt, Clone, Debug)]
pub(crate) enum SomeImportStrategy {
    #[structopt(about = "Create namespace from a json collection")]
    FromFile(FileImportStrategy),
    #[structopt(about = "Create namespace from a postgres db")]
    FromPostgres(PostgresImportStrategy),
}

#[derive(StructOpt, Clone, Debug)]
pub(crate) struct PostgresImportStrategy {
    uri: String,
}

#[derive(StructOpt, Clone, Debug)]
pub(crate) struct FileImportStrategy {
    #[structopt(parse(from_os_str))]
    from_file: PathBuf,
}

impl ImportStrategy for SomeImportStrategy {
    fn import(self) -> Result<Namespace> {
        match self {
            SomeImportStrategy::FromFile(fis) => fis.import(),
            SomeImportStrategy::FromPostgres(pis) => pis.import(),
        }
    }
}

impl ImportStrategy for FileImportStrategy {
    fn import(self) -> Result<Namespace> {
        let _buff = std::fs::read_to_string(self.from_file)?;
        unimplemented!()
    }
}

impl ImportStrategy for PostgresImportStrategy {
    fn import(self) -> Result<Namespace> {
        unimplemented!("Postgres is not supported yet")
    }
}
