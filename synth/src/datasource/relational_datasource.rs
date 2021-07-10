use crate::datasource::DataSource;
use serde_json::Value;
use anyhow::{Result};
use async_trait::async_trait;
use futures::future::join_all;
use beau_collector::BeauCollector;

const DEFAULT_INSERT_BATCH_SIZE: usize = 1000;

#[derive(Debug)]
pub struct ColumnInfo {
    pub(crate) column_name: String,
    pub(crate) ordinal_position: i32,
    pub(crate) is_nullable: bool,
    pub(crate) data_type: String,
    pub(crate) character_maximum_length: Option<i32>,
}

#[derive(Debug)]
pub struct PrimaryKey {
    pub(crate) column_name: String,
    pub(crate) type_name: String,
}

#[derive(Debug)]
pub struct ForeignKey {
    pub(crate) from_table: String,
    pub(crate) from_column: String,
    pub(crate) to_table: String,
    pub(crate) to_column: String,
}

#[derive(Debug)]
pub struct ValueWrapper(pub(crate) Value);

#[async_trait]
pub trait RelationalDataSource : DataSource {
    type QueryResult: Send + Sync;

    async fn insert_relational_data(&self, collection_name: String, collection: &[Value]) -> Result<()> {
        // how to to ordering here?
        // If we have foreign key constraints we need to traverse the tree and
        // figure out insertion order
        // We basically need something like an InsertionStrategy where we have a DAG of insertions
        let batch_size = DEFAULT_INSERT_BATCH_SIZE;

        if collection.is_empty() {
            println!(
                "Collection {} generated 0 values. Skipping insertion...",
                collection_name
            );
            return Ok(());
        }

        let column_names = collection
            .get(0)
            .expect("Explicit check is done above that this collection is non-empty")
            .as_object()
            .expect("This is always an object (sampler contract)")
            .keys()
            .cloned()
            .collect::<Vec<String>>()
            .join(",");

        let mut futures = vec![];

        for rows in collection.chunks(batch_size) {
            let mut query = format!(
                "INSERT INTO {} ({}) VALUES \n",
                collection_name, column_names
            );

            for (i, row) in rows.iter().enumerate() {
                let row_obj = row
                    .as_object()
                    .expect("This is always an object (sampler contract)");

                let values = row_obj
                    .values()
                    .map(|v| v.to_string().replace("'", "''")) // two single quotes are the standard way to escape double quotes
                    .map(|v| v.replace("\"", "'")) // values in the object have quotes around them by default.
                    .collect::<Vec<String>>()
                    .join(",");

                // We should be using some form of a prepared statement here.
                // It is not clear how this would work for our batch inserts...
                if i == rows.len() - 1 {
                    query.push_str(&format!("({});\n", values));
                } else {
                    query.push_str(&format!("({}),\n", values));
                }
            }

            let future = self.execute_query(query, vec![]);
            futures.push(future);
        }

        let results: Vec<Result<Self::QueryResult>> = join_all(futures).await;

        if let Err(e) = results.into_iter().bcollect::<Vec<Self::QueryResult>>() {
            bail!("One or more database inserts failed: {:?}", e)
        }

        println!("Inserted {} rows...", collection.len());
        Ok(())
    }

    async fn execute_query(&self, query: String, query_params: Vec<&str>) -> Result<Self::QueryResult>;

    fn get_catalog(&self) -> Result<&str>;

    async fn get_table_names(&self) -> Result<Vec<String>>;

    async fn get_columns_infos(&self, table_name: &str) -> Result<Vec<ColumnInfo>>;

    async fn get_primary_keys(&self, table_name: &str) -> Result<Vec<PrimaryKey>>;

    async fn get_foreign_keys(&self) -> Result<Vec<ForeignKey>>;

    async fn get_deterministic_samples(&self, table_name: &str) -> Result<Vec<Value>>;
}