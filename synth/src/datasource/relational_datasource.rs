use crate::datasource::DataSource;
use anyhow::Result;
use async_trait::async_trait;
use beau_collector::BeauCollector;
use futures::future::join_all;
use synth_core::{Content, Value};
use synth_gen::value::Number;

const DEFAULT_INSERT_BATCH_SIZE: usize = 1000;

//TODO: Remove this once https://github.com/rust-lang/rust/issues/88900 gets fixed
#[allow(dead_code)]
#[derive(Debug)]
pub struct ColumnInfo {
    pub(crate) column_name: String,
    pub(crate) ordinal_position: i32,
    pub(crate) is_nullable: bool,
    pub(crate) data_type: String,
    pub(crate) character_maximum_length: Option<i32>,
}

#[allow(dead_code)]
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

    async fn insert_relational_data(
        &self,
        collection_name: String,
        collection: &[Value],
    ) -> Result<()> {
        let batch_size = DEFAULT_INSERT_BATCH_SIZE;

        if collection.is_empty() {
            println!(
                "Collection {} generated 0 values. Skipping insertion...",
                collection_name
            );
            return Ok(());
        }

        let column_infos = self.get_columns_infos(&collection_name).await?;
        let first_valueset = collection
            .get(0)
            .expect("Explicit check is done above that this collection is non-empty")
            .as_object()
            .expect("This is always an object (sampler contract)");

        for column_info in column_infos {
            if let Some(value) = first_valueset.get(&column_info.column_name) {
                match (value, &*column_info.data_type) {
                    (
                        Value::Number(Number::U64(_)),
                        "int2" | "int4" | "int8" | "int" | "integer" | "smallint" | "bigint",
                    ) => warn!(
                        "Trying to put an unsigned u64 into a {} typed column {}.{}",
                        column_info.data_type, collection_name, column_info.column_name
                    ),
                    (
                        Value::Number(Number::U32(_)),
                        "int2" | "int4" | "int8" | "int" | "integer" | "smallint" | "bigint",
                    ) => warn!(
                        "Trying to put an unsigned u32 into a {} typed column {}.{}",
                        column_info.data_type, collection_name, column_info.column_name
                    ),
                    //TODO: More variants
                    _ => {}
                }
            }
        }

        let column_names = first_valueset
            .keys()
            .cloned()
            .collect::<Vec<String>>()
            .join(",");

        let mut futures = Vec::with_capacity(collection.len());

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
            let future = self.execute_query(query, query_params);
            futures.push(future);
        }

        let results: Vec<Result<Self::QueryResult>> = join_all(futures).await;

        if let Err(e) = results.into_iter().bcollect::<Vec<Self::QueryResult>>() {
            bail!("One or more database inserts failed: {:?}", e)
        }

        info!("Inserted {} rows...", collection.len());
        Ok(())
    }

    async fn execute_query(
        &self,
        query: String,
        query_params: Vec<&Value>,
    ) -> Result<Self::QueryResult>;

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
