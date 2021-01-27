use std::convert::AsRef;
use std::fs::File;
use std::path::{Path, PathBuf};

use diesel::{sqlite::SqliteConnection, Connection, ExpressionMethods, RunQueryDsl};

use diesel::result::Error as DieselError;

use chrono::{NaiveDateTime, Utc};

use anyhow::{Context, Result};

use crate::index::generations::dsl::*;
use crate::store::FileStore;
use synth_core::{
    error::{Error, ErrorKind},
    schema::{Name, Namespace},
};

use fs2::FileExt;

embed_migrations!("migrations/");

table! {
    generations (namespace, generation) {
    namespace -> Text,
    generation -> Integer,
    timestamp -> Timestamp,
    }
}

pub struct Index {
    store: FileStore,
    conn_str: String,
}

#[derive(QueryableByName, Queryable, Insertable)]
#[table_name = "generations"]
#[derive(Clone, Debug)]
pub struct NamespaceEntry {
    pub namespace: String,
    pub generation: i32,
    pub timestamp: NaiveDateTime,
}

impl NamespaceEntry {
    pub fn new(name: &Name) -> Self {
        let now = NaiveDateTime::from_timestamp(Utc::now().timestamp(), 0);
        Self {
            namespace: name.to_string(),
            generation: 0,
            timestamp: now,
        }
    }

    fn generation_path(&self) -> PathBuf {
        self.at_generation(self.generation)
    }

    fn at_generation(&self, gen: i32) -> PathBuf {
        format!("{}/{}", self.namespace, gen).into()
    }

    fn namespace_path(&self) -> PathBuf {
        format!("{}/", self.namespace).into()
    }

    fn lock_path(&self) -> PathBuf {
        format!("{}/.lock", self.namespace).into()
    }

    fn update(&self) -> Self {
        let now = NaiveDateTime::from_timestamp(Utc::now().timestamp(), 0);
        Self {
            namespace: self.namespace.clone(),
            generation: self.generation + 1,
            timestamp: now,
        }
    }
}

impl Index {
    pub fn at<R: AsRef<Path>>(path: R) -> Result<Self> {
        let expanded_path = path.as_ref().canonicalize()?;
        debug!("index directory root: {}", expanded_path.to_str().unwrap());

        let conn_str = Index::build_conn_str(&expanded_path);
        let store = FileStore::new(expanded_path)?;
        let out = Self { store, conn_str };
        out.verify()?;
        Ok(out)
    }

    fn build_conn_str<R: AsRef<Path>>(repo: R) -> String {
        let mut path = repo.as_ref().to_owned();
        path.push("db.sqlite");
        path.to_str()
            .expect("could not create path for db.sqlite")
            .to_string()
    }

    pub fn create_ns(&self, ns: &Name) -> Result<()> {
        let conn = self.conn()?;
        conn.transaction::<_, DieselError, _>(|| {
            let nse = NamespaceEntry::new(ns);
            diesel::insert_into(generations::table)
                .values(&nse)
                .execute(&conn)?;
            self.store.init(nse.namespace_path()).unwrap();
            // Dropping the `_lock` releases the OS level lock as well
            let _lock = self
                .store
                .lock_exclusive(nse.lock_path())
                .map_err(|_| DieselError::RollbackTransaction)?;
            self.store
                .insert(nse.generation_path(), Namespace::default())
                .map_err(|_| DieselError::RollbackTransaction)?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn list_ns(&self) -> Result<Vec<NamespaceEntry>> {
        let query = format!("SELECT * FROM generations");
        let nss: Vec<NamespaceEntry> = diesel::sql_query(query).load(&self.conn()?)?;
        Ok(nss)
    }

    pub fn delete_ns(&self, ns: &Name) -> Result<()> {
        let entry = self.ns_entry_for_name(ns)?;
        let conn = self.conn()?;
        conn.transaction::<_, DieselError, _>(|| {
            diesel::delete(generations)
                .filter(namespace.eq(ns.to_string()))
                .execute(&conn)?;
            self.store
                .lock_exclusive(entry.lock_path())
                .and_then(|_| {
                    self.store
                        .ls(entry.namespace_path())?
                        .try_for_each(|generation_path| self.store.delete(generation_path))
                })
                .map_err(|_| DieselError::RollbackTransaction)?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn rollback_ns(&self, ns_name: &Name, gen: i32) -> Result<()> {
        let ns_entry = self.ns_entry_for_name(ns_name)?;

        let _lock = self
            .store
            .lock_exclusive(ns_entry.lock_path())
            .context(anyhow!(
                "while acquiring a shared lock on the namespace: {}",
                ns_name
            ))?;

        let old_ns = self
            .store
            .get(ns_entry.at_generation(gen))
            .context(anyhow!(
                "while retrieving schema generation '{}' for namespace '{}'",
                gen,
                ns_entry.namespace
            ))?;

        self.commit(&ns_entry, old_ns)
    }

    #[allow(dead_code)]
    pub fn exists(&self, ns: &Name) -> Result<bool> {
        self.borrow(ns)
            .map(|_| true)
            .or_else(|err| match err.downcast_ref::<Error>() {
                Some(err) if *err.kind() == ErrorKind::NotFound => Ok(true),
                _ => Err(err),
            })
    }

    pub fn borrow_at_gen(&self, ns_name: &Name, gen: Option<i32>) -> Result<IndexGuard> {
        match gen {
            Some(gen) => self.borrow_inner(self.ns_entry_for_name(ns_name)?, gen),
            None => self.borrow(ns_name),
        }
    }

    pub fn borrow(&self, ns_name: &Name) -> Result<IndexGuard> {
        let ns_entry = self.ns_entry_for_name(ns_name)?;
        let gen = ns_entry.generation;
        self.borrow_inner(ns_entry, gen)
    }

    fn borrow_inner(&self, ns_entry: NamespaceEntry, gen: i32) -> Result<IndexGuard> {
        let lock = self
            .store
            .lock_shared(ns_entry.lock_path())
            .context(anyhow!(
                "while acquiring a shared lock on the namespace: {}",
                ns_entry.namespace
            ))?;
        let ns: Namespace = self
            .store
            .get(ns_entry.at_generation(gen))
            .context(anyhow!(
                "while retrieving schema for namespace '{}'",
                &ns_entry.namespace,
            ))?;
        Ok(IndexGuard::shared(self, ns_entry, ns, lock))
    }

    pub fn borrow_mut(&self, ns_name: &Name) -> Result<IndexGuardMut> {
        let ns_entry = self.ns_entry_for_name(ns_name)?;
        let lock = self
            .store
            .lock_exclusive(ns_entry.lock_path())
            .context(anyhow!(
                "while acquiring an exclusive lock on the namespace: {}",
                ns_name
            ))?;
        let ns: Namespace = self.store.get(ns_entry.generation_path()).context(anyhow!(
            "while retrieving schema for namespace '{}'",
            ns_name,
        ))?;
        Ok(IndexGuardMut::exclusive(self, ns_entry, ns, lock))
    }

    fn conn(&self) -> Result<SqliteConnection> {
        Ok(SqliteConnection::establish(&self.conn_str)?)
    }

    fn verify(&self) -> Result<()> {
        embedded_migrations::run(&self.conn()?)?;
        Ok(())
    }

    fn ns_entry_for_name(&self, ns: &Name) -> Result<NamespaceEntry> {
        let query = format!("SELECT * FROM generations WHERE namespace = \"{}\"", ns);
        let nss: Vec<NamespaceEntry> = diesel::sql_query(query).load(&self.conn()?)?;
        nss.into_iter()
            .max_by_key(|item| item.generation)
            .ok_or(failed!(target: Release, NotFound => "no such namespace: {}", ns))
    }

    fn commit(&self, nse: &NamespaceEntry, ns: Namespace) -> Result<()> {
        let conn = self.conn()?;
        conn.transaction::<_, DieselError, _>(|| {
            let nse = nse.update();
            let generation_path = nse.generation_path();
            diesel::insert_into(generations)
                .values(vec![nse])
                .execute(&conn)?;
            self.store
                .insert(generation_path, ns)
                .map_err(|_| DieselError::RollbackTransaction)?;
            Ok(())
        })?;
        Ok(())
    }
}

struct InnerIndexGuard<'a> {
    index: &'a Index,
    ns_entry: NamespaceEntry,
    namespace: Namespace,
    lock: IndexLock,
}

enum IndexLock {
    Exclusive { lock_file: File },
    Shared { lock_file: File },
}

impl IndexLock {
    fn unlock(&self) -> Result<()> {
        match self {
            IndexLock::Exclusive { lock_file: lock } | IndexLock::Shared { lock_file: lock } => {
                lock.unlock().map_err(|e| e.into())
            }
        }
    }
}

pub struct IndexGuard<'a>(InnerIndexGuard<'a>);

impl<'a> AsRef<Namespace> for IndexGuard<'a> {
    fn as_ref(&self) -> &Namespace {
        &self.0.namespace
    }
}

impl<'a> std::ops::Deref for IndexGuard<'a> {
    type Target = Namespace;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> IndexGuard<'a> {
    fn shared(index: &'a Index, ns_entry: NamespaceEntry, inner: Namespace, lock: File) -> Self {
        Self(InnerIndexGuard {
            index,
            ns_entry,
            namespace: inner,
            lock: IndexLock::Shared { lock_file: lock },
        })
    }
}

pub struct IndexGuardMut<'a>(InnerIndexGuard<'a>);

impl<'a> AsRef<Namespace> for IndexGuardMut<'a> {
    fn as_ref(&self) -> &Namespace {
        &self.0.namespace
    }
}

impl<'a> AsMut<Namespace> for IndexGuardMut<'a> {
    fn as_mut(&mut self) -> &mut Namespace {
        &mut self.0.namespace
    }
}

impl<'a> std::ops::Deref for IndexGuardMut<'a> {
    type Target = Namespace;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> std::ops::DerefMut for IndexGuardMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<'a> IndexGuardMut<'a> {
    fn exclusive(index: &'a Index, ns_entry: NamespaceEntry, inner: Namespace, lock: File) -> Self {
        Self(InnerIndexGuard {
            index,
            ns_entry,
            namespace: inner,
            lock: IndexLock::Exclusive { lock_file: lock },
        })
    }

    pub fn commit(self) -> Result<()> {
        let inner = self.0;
        match inner.lock {
            IndexLock::Exclusive { .. } => {
                inner
                    .index
                    .commit(&inner.ns_entry, inner.namespace.clone())?;
            }
            _ => panic!("attempted to commit mutations through a shared lock"),
        }
        inner.lock.unlock()?;
        Ok(())
    }
}

impl Drop for InnerIndexGuard<'_> {
    fn drop(&mut self) {
        self.lock.unlock().unwrap();
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use tempfile::tempdir;

    #[test]
    fn test_index() -> Result<()> {
        let data_directory = tempdir().unwrap();

        let index = Index::at(data_directory.path()).unwrap();
        let namespace_name = "some_namespace".parse().unwrap();
        let collection_name = "some_collection".parse().unwrap();

        index.create_ns(&namespace_name)?;

        {
            let mut namespace_ = index.borrow_mut(&namespace_name)?;
            println!("{:?}", namespace_.clone());
            namespace_.create_collection(
                &collection_name,
                &json!({
                            "a_field": 5
                }),
            )?;

            println!("{:?}", namespace_.clone());

            namespace_.commit()?;
        }

        {
            let namespace_ = index.borrow(&namespace_name)?;
            namespace_.get_collection(&collection_name)?;
            println!("{:?}", namespace);
        }

        index.delete_ns(&namespace_name)?;

        assert!(index.borrow(&namespace_name).is_err());

        Ok(())
    }

    #[test]
    fn test_index_ls() -> Result<()> {
        let data_directory = tempdir().unwrap();

        let index = Index::at(data_directory.path()).unwrap();
        let namespace_name = "some_namespace".parse().unwrap();
        let collection_name = "some_collection".parse().unwrap();

        index.create_ns(&namespace_name)?;

        {
            let mut namespace_ = index.borrow_mut(&namespace_name)?;
            namespace_.create_collection(
                &collection_name,
                &json!({
                            "a_field": 1
                }),
            )?;

            namespace_.commit()?;
        }

        let gens = index.list_ns().unwrap();

        let gen_1 = gens.get(0).unwrap();
        assert_eq!(gen_1.namespace, namespace_name.to_string());
        assert_eq!(gen_1.generation, 0);

        let gen_2 = gens.get(1).unwrap();
        assert_eq!(gen_2.namespace, namespace_name.to_string());
        assert_eq!(gen_2.generation, 1);

        assert!(gen_1.timestamp <= gen_2.timestamp);

        Ok(())
    }

    #[test]
    fn test_index_delete_ns() -> Result<()> {
        let data_directory = tempdir().unwrap();

        let index = Index::at(data_directory.path()).unwrap();
        let namespace_name = "some_namespace".parse().unwrap();

        index.create_ns(&namespace_name)?;
        index.delete_ns(&namespace_name)?;
        assert!(index.list_ns().unwrap().is_empty());
        Ok(())
    }

    #[test]
    fn test_rollback_ns() -> Result<()> {
        let data_directory = tempdir().unwrap();

        let index = Index::at(data_directory.path()).unwrap();
        let namespace_name = "some_namespace".parse().unwrap();

        let gen_1;

        index.create_ns(&namespace_name)?;
        {
            let mut namespace_ = index.borrow_mut(&namespace_name)?;
            namespace_.create_collection(
                &"some_collection".parse().unwrap(),
                &json!({
                            "a_field": 1
                }),
            )?;
            gen_1 = namespace_.collections.clone();
            namespace_.commit()?;
        }

        {
            let mut namespace_ = index.borrow_mut(&namespace_name)?;
            namespace_.create_collection(
                &"some_other_collection".parse().unwrap(),
                &json!({
                            "another_field": 1
                }),
            )?;

            namespace_.commit()?;
        }

        index.rollback_ns(&namespace_name, 1)?;

        let gens = index.list_ns().unwrap();
        assert_eq!(gens.len(), 4);
        assert_eq!(gen_1, index.borrow(&namespace_name).unwrap().collections);
        Ok(())
    }
}
