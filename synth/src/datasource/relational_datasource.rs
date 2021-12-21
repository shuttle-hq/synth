use crate::datasource::DataSource;
use anyhow::Result;
use async_trait::async_trait;
use beau_collector::BeauCollector;
use futures::future::join_all;
use sqlx::{query::Query, Arguments, Database, Executor, IntoArguments, Pool};
use std::convert::TryFrom;
use synth_core::{Content, Value};
use synth_gen::value::Number;

const DEFAULT_INSERT_BATCH_SIZE: usize = 1000;

//TODO: Remove this once https://github.com/rust-lang/rust/issues/88900 gets fixed
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub(crate) column_name: String,
    pub(crate) ordinal_position: i32,
    pub(crate) is_nullable: bool,
    pub(crate) is_custom_type: bool,
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

#[async_trait]
pub trait SqlxDataSource: DataSource {
    type DB: Database<Arguments = Self::Arguments>;
    type Arguments: for<'q> Arguments<'q, Database = Self::DB> + for<'q> IntoArguments<'q, Self::DB>;

    /// Gets a pool to execute queries with
    fn get_pool(&self) -> Pool<Self::DB>;

    /// Prepare a single query with data source specifics
    fn query<'q>(&self, query: &'q str) -> Query<'q, Self::DB, Self::Arguments>;

    /// Get query for table names
    fn get_table_names_query(&self) -> &str;

    /// Get query for primary keys
    fn get_primary_keys_query(&self) -> &str;

    /// Get query for foreign keys
    fn get_foreign_keys_query(&self) -> &str;

    /// Get query for columns info
    fn get_columns_info_query(&self) -> &str;

    async fn set_seed(&self) -> Result<()> {
        // Default for sources that don't need to set a seed
        Ok(())
    }

    /// Get query for deterministic values
    fn get_deterministic_samples_query(&self, table_name: String) -> String;

    /// Decodes column to our Content
    fn decode_to_content(&self, column_info: &ColumnInfo) -> Result<Content>;

    // Returns extended query string + current index
    fn extend_parameterised_query(
        query: &mut String,
        _curr_index: usize,
        query_params: Vec<Value>,
    ) {
        let extend = query_params.len();

        query.push('(');
        for i in 0..extend {
            query.push('?');
            if i != extend - 1 {
                query.push(',');
            }
        }
        query.push(')');
    }
}

/// All relational databases should define this trait and implement database specific queries in
/// their own impl. APIs should be defined async when possible, delegating to the caller on how to
/// handle it.
#[async_trait]
pub trait RelationalDataSource: DataSource + SqlxDataSource
where
    Self: Sized,
    for<'c> &'c mut <Self::DB as Database>::Connection: Executor<'c, Database = Self::DB>,
    String: sqlx::Type<Self::DB>,
    for<'d> String: sqlx::Encode<'d, Self::DB>,
    ColumnInfo: TryFrom<<Self::DB as sqlx::Database>::Row, Error = anyhow::Error>,
{
    type QueryResult: Send + Sync;

    const IDENTIFIER_QUOTE: char;

    async fn insert_relational_data(
        &self,
        collection_name: &str,
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

        let column_infos = get_columns_info(self, collection_name.to_string()).await?;
        let first_valueset = collection[0]
            .as_object()
            .expect("This is always an object (sampler contract)");

        for column_info in column_infos {
            if let Some(value) = first_valueset.get(&column_info.column_name) {
                match (value, &*column_info.data_type) {
                    (
                        Value::Number(Number::U64(_)),
                        "int2" | "int4" | "int8" | "smallint" | "int" | "bigint",
                    ) => warn!(
                        "Trying to put an unsigned u64 into a {} typed column {}.{}",
                        column_info.data_type, collection_name, column_info.column_name
                    ),
                    (
                        Value::Number(Number::U32(_)),
                        "int2" | "int4" | "int8" | "smallint" | "int" | "bigint",
                    ) => warn!(
                        "Trying to put an unsigned u32 into a {} typed column {}.{}",
                        column_info.data_type, collection_name, column_info.column_name
                    ),
                    (Value::Number(Number::I64(_)), "int2" | "int4" | "smallint" | "int") => warn!(
                        "Trying to put a signed i64 into a {} typed column {}.{}",
                        column_info.data_type, collection_name, column_info.column_name
                    ),
                    (Value::Number(Number::I32(_)), "int2" | "int8" | "smallint" | "bigint") => {
                        warn!(
                            "Trying to put a signed i32 into a {} typed column {}.{}",
                            column_info.data_type, collection_name, column_info.column_name
                        )
                    }
                    //TODO: More variants
                    _ => {}
                }
            }
        }

        let column_names = first_valueset
            .keys()
            .map(|k| format!("{}{}{}", Self::IDENTIFIER_QUOTE, k, Self::IDENTIFIER_QUOTE))
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

                let mut curr_query_params: Vec<Value> = row_obj.values().cloned().collect();
                Self::extend_parameterised_query(&mut query, curr_index, curr_query_params.clone());
                curr_index += curr_query_params.len();
                query_params.append(&mut curr_query_params);

                if i == rows.len() - 1 {
                    query.push_str(";\n");
                } else {
                    query.push_str(",\n");
                }
            }
            let future = self.execute_query(query, query_params);
            futures.push(future);
        }

        let results = join_all(futures).await;

        if let Err(e) = results.into_iter().bcollect::<Vec<Self::QueryResult>>() {
            bail!("One or more database inserts failed: {:?}", e)
        }

        info!("Inserted {} rows...", collection.len());
        Ok(())
    }

    async fn execute_query(
        &self,
        query: String,
        query_params: Vec<Value>,
    ) -> Result<Self::QueryResult>;
}

pub async fn get_columns_info<T: SqlxDataSource>(
    datasource: &T,
    table_name: String,
) -> Result<Vec<ColumnInfo>>
where
    for<'c> &'c mut <T::DB as Database>::Connection: Executor<'c, Database = T::DB>,
    String: sqlx::Type<T::DB>,
    for<'d> String: sqlx::Encode<'d, T::DB>,
    ColumnInfo: TryFrom<<T::DB as sqlx::Database>::Row, Error = anyhow::Error>,
{
    let query = datasource.get_columns_info_query();
    let pool = datasource.get_pool();

    datasource
        .query(query)
        .bind(table_name)
        .fetch_all(&pool)
        .await?
        .into_iter()
        .map(ColumnInfo::try_from)
        .collect()
}
