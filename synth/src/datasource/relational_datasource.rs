use crate::datasource::DataSource;
use anyhow::{Result};
use async_trait::async_trait;
use futures::future::join_all;
use beau_collector::BeauCollector;
use synth_core::Content;
use synth_core::Value;

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

/// Wrapper around `Value` since we can't impl `TryFrom` on a struct in a non-owned crate
#[derive(Debug)]
pub struct ValueWrapper(pub(crate) Value);

/// All relational databases should define this trait and implement database specific queries in
/// their own impl. APIs should be defined async when possible, delegating to the caller on how to
/// handle it.
#[async_trait]
pub trait RelationalDataSource: DataSource {
    type QueryResult: Send + Sync;

    async fn insert_relational_data(&self, collection_name: String, collection: &[Value]) -> Result<()> {
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

            let mut curr_index = 0;
            let mut query_params = vec![];

            for (i, row) in rows.iter().enumerate() {
                let row_obj = row
                    .as_object()
                    .expect("This is always an object (sampler contract)");
                let extend = row_obj.values().len();
                Self::extend_parameterised_query(&mut query, curr_index, extend);
                curr_index += extend;
                query_params.extend(row_obj.values());

                if i == rows.len() - 1 {
                    query.push_str(";\n");
                } else {
                    query.push_str(",\n");
                }

            }
            println!("{}", query);
            println!("{:#?}", query_params);
            let future = self.execute_query(query, query_params);
            futures.push(future);
        }

        let results: Vec<Result<Self::QueryResult>> = join_all(futures).await;

        if let Err(e) = results.into_iter().bcollect::<Vec<Self::QueryResult>>() {
            bail!("One or more database inserts failed: {:?}", e)
        }

        println!("Inserted {} rows...", collection.len());
        Ok(())
    }

    async fn execute_query(&self, query: String, query_params: Vec<&Value>) -> Result<Self::QueryResult>;

    fn get_catalog(&self) -> Result<&str>;

    async fn get_table_names(&self) -> Result<Vec<String>>;

    async fn get_columns_infos(&self, table_name: &str) -> Result<Vec<ColumnInfo>>;

    async fn get_primary_keys(&self, table_name: &str) -> Result<Vec<PrimaryKey>>;

    async fn get_foreign_keys(&self) -> Result<Vec<ForeignKey>>;

    async fn set_seed(&self) -> Result<()>;

    async fn get_deterministic_samples(&self, table_name: &str) -> Result<Vec<Value>>;

    fn decode_to_content(&self, data_type: &str, _char_max_len: Option<i32>) -> Result<Content>;

    // Returns extended query string + current index
    fn extend_parameterised_query(query: &mut String, curr_index: usize, extend: usize);
}