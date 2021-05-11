use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum StoreError {
    Missing,
    Corrupted { msg: String },
    WriteError,
    FileError { msg: String },
    LockError { msg: String },
    InitialisationError { msg: String },
}

impl Display for StoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Missing => write!(f, "missing file"),
            StoreError::Corrupted { msg } => write!(f, "corrupted file: {}", msg),
            StoreError::WriteError => write!(f, "write error"),
            StoreError::FileError { msg } => write!(f, "file error: {}", msg),
            StoreError::LockError { msg } => write!(f, "lock file: {}", msg),
            StoreError::InitialisationError { msg } => write!(f, "corrupted file: {}", msg),
        }
    }
}

impl std::error::Error for StoreError {}

pub struct FileStore {
    repo: PathBuf,
}

impl FileStore {
    pub fn new(repo: PathBuf) -> anyhow::Result<Self> {
        fs::create_dir_all(&repo)?;
        Ok(FileStore { repo })
    }

    pub fn get<P: AsRef<Path>, T>(&self, location: P) -> Result<T, StoreError>
    where
        for<'a> T: Deserialize<'a>,
    {
        let file_ref = self.repo.join(location);
        let contents = File::open(file_ref)
            .map_err(|_| StoreError::Missing)
            .and_then(|mut file| {
                let mut contents = String::new();
                file.read_to_string(&mut contents)
                    .map_err(|err| StoreError::FileError {
                        msg: err.to_string(),
                    })?;
                Ok(contents)
            })?;
        let ty: T = serde_json::from_str(&contents)
            .map_err(|e| StoreError::Corrupted { msg: e.to_string() })?;
        Ok(ty)
    }

    pub fn insert<P: AsRef<Path>, T: Serialize>(
        &self,
        location: P,
        t: T,
    ) -> Result<(), StoreError> {
        let file_ref = self.repo.join(location);
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_ref)
            .map_err(|e| StoreError::FileError { msg: e.to_string() })?;
        let as_string = serde_json::to_string(&t).unwrap(); // todo
        file.write_all(as_string.as_bytes())
            .map_err(|_| StoreError::WriteError)?;
        Ok(())
    }

    pub fn ls<P: AsRef<Path>>(
        &'_ self,
        location: P,
    ) -> Result<impl Iterator<Item = PathBuf> + '_, StoreError> {
        fs::read_dir(self.repo.join(location))
            .and_then(|read_dir| {
                let out = read_dir
                    .collect::<io::Result<Vec<_>>>()?
                    .into_iter()
                    .filter(|dir_entry| {
                        dir_entry
                            .file_name()
                            .to_str()
                            .map(|fn_| fn_.chars().all(char::is_numeric))
                            .unwrap_or(false)
                    })
                    .map(move |dir_entry| {
                        dir_entry
                            .path()
                            .strip_prefix(&self.repo)
                            .unwrap()
                            .to_path_buf()
                    });
                Ok(out)
            })
            .map_err(|err| StoreError::FileError {
                msg: err.to_string(),
            })
    }

    pub fn delete<P: AsRef<Path>>(&self, location: P) -> Result<(), StoreError> {
        fs::remove_file(self.repo.join(location)).map_err(|err| StoreError::FileError {
            msg: err.to_string(),
        })
    }

    pub fn init<P: AsRef<Path>>(&self, location: P) -> Result<(), StoreError> {
        let mut fd = self.repo.join(location);

        // Create directory
        fs::create_dir_all(fd.clone()).map_err(|e| StoreError::InitialisationError {
            msg: format!(
                "Could not create directory {:?} with error: {}",
                fd,
                e.to_string()
            ),
        })?;

        // Create lock file
        fd.push(".lock");

        OpenOptions::new()
            .create(true)
            .write(true)
            .open(fd)
            .map_err(|e| StoreError::FileError { msg: e.to_string() })?;

        Ok(())
    }

    pub fn lock_shared<P: AsRef<Path>>(&self, location: P) -> Result<File, StoreError> {
        let file_ref = self.repo.join(location);
        let file = File::open(file_ref).map_err(|_| StoreError::Missing)?;
        file.lock_shared()
            .map_err(|e| StoreError::LockError { msg: e.to_string() })?;
        Ok(file)
    }

    pub fn lock_exclusive<P: AsRef<Path>>(&self, location: P) -> Result<File, StoreError> {
        let file_ref = self.repo.join(location);
        let file = File::open(file_ref).map_err(|_| StoreError::Missing)?;
        file.lock_exclusive()
            .map_err(|e| StoreError::LockError { msg: e.to_string() })?;
        Ok(file)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;

    use std::ffi::OsString;

    #[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
    struct TestStruct {
        inner: String,
    }

    #[test]
    fn test_rw() {
        let a_ref = "a".parse::<OsString>().unwrap();
        let a = TestStruct {
            inner: "a".to_string(),
        };

        let b_ref = "b".parse::<OsString>().unwrap();
        let b = TestStruct {
            inner: "b".to_string(),
        };

        let dir = tempdir().unwrap();
        let store = FileStore::new(dir.path().into()).unwrap();

        // RW A
        store.insert(&a_ref, a.clone()).unwrap();
        assert_eq!(&a, &store.get(&a_ref).unwrap());

        // RW B
        store.insert(&b_ref, b.clone()).unwrap();
        assert_eq!(&b, &store.get(&b_ref).unwrap());

        // Over-write B with A
        store.insert(&b_ref, a.clone()).unwrap();
        assert_eq!(&a, &store.get(&b_ref).unwrap());
    }

    #[test]
    fn test_e2e() {
        let namespace_name = "some_ns".parse::<OsString>().unwrap();
        let dir = tempdir().unwrap();
        let store = FileStore::new(dir.path().into()).unwrap();

        store.init(&namespace_name).unwrap();
        let lock = store.lock_exclusive(&namespace_name).unwrap();

        let struct_ref = "some_ns/some_struct".parse::<OsString>().unwrap();
        let some_struct = TestStruct {
            inner: "some_text".to_string(),
        };

        store.insert(&struct_ref, some_struct.clone()).unwrap();
        assert_eq!(&some_struct, &store.get(&struct_ref).unwrap());

        drop(lock);
    }
}
