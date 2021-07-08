use crate::datasource::DataSource;
use anyhow::Result;
use crate::datasource::relational_datasource::{RelationalDataSource, ColumnInfo, PrimaryKey, ForeignKey};
use sqlx::{Pool, Postgres, Row};
use sqlx::postgres::{PgPoolOptions, PgQueryResult, PgRow};
use serde_json::Value;
use async_std::task;
use async_trait::async_trait;
use std::convert::TryFrom;

pub struct PostgresDataSource {
    pool: Pool<Postgres>,
    connect_params: String
}

#[async_trait]
impl DataSource for PostgresDataSource {
    type ConnectParams = String;

    fn new(connect_params: &Self::ConnectParams) -> Result<Self> {
        task::block_on(async {
            let pool = PgPoolOptions::new()
                .max_connections(3) //TODO expose this as a user config?
                .connect(connect_params.as_str())
                .await?;

            Ok::<Self, anyhow::Error>(PostgresDataSource {
                pool,
                connect_params: connect_params.to_string()
            })
        })
    }

    async fn insert_data(&self, collection_name: String, collection: &[Value]) -> Result<()> {
        self.insert_relational_data(collection_name, collection).await.unwrap();
        Ok(())
    }
}

#[async_trait]
impl RelationalDataSource for PostgresDataSource {
    type QueryResult = PgQueryResult;

    async fn execute_query(&self, query: String, _query_params: Vec<&str>) -> Result<PgQueryResult> {
        let result = sqlx::query(query.as_str())
            .execute(&self.pool)
            .await?;

        Ok(result)
    }

    fn get_catalog(&self) -> Result<&str> {
        self.connect_params
            .split('/')
            .last()
            .ok_or_else(|| anyhow!("No catalog specified in the uri"))
    }

    async fn get_table_names(&self) -> Result<Vec<String>> {
        let query = r"SELECT table_name
        FROM information_schema.tables
        WHERE table_catalog = $1 AND table_schema = 'public' AND table_type = 'BASE TABLE'";

        sqlx::query(query)
            .bind(self.get_catalog()?)
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(|row| row.try_get::<String, usize>(0).map_err(|e| anyhow!("{:?}", e)))
            .collect()
    }

    async fn get_columns_infos(&self, table_name: &str) -> Result<Vec<ColumnInfo>> {
        let query = r"SELECT column_name, ordinal_position, is_nullable, udt_name,
        character_maximum_length
        FROM information_schema.columns
        WHERE table_name = $1 AND table_catalog = $2";

        sqlx::query(query)
            .bind(table_name)
            .bind(self.get_catalog()?)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(ColumnInfo::try_from)
            .collect::<Result<Vec<ColumnInfo>>>()
    }

    async fn get_primary_keys(&self, table_name: &str) -> Result<Vec<PrimaryKey>> {
        // Unfortunately cannot use parameterised queries here
        let query: &str = &format!(
            r"SELECT a.attname, format_type(a.atttypid, a.atttypmod) AS data_type
            FROM pg_index i
            JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey)
            WHERE  i.indrelid = '{}'::regclass AND i.indisprimary",
            &table_name
        );

        sqlx::query(query)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(PrimaryKey::try_from)
            .collect::<Result<Vec<PrimaryKey>>>()
    }

    async fn get_foreign_keys(&self) -> Result<Vec<ForeignKey>> {
        let query: &str = 
            r"SELECT tc.table_name, kcu.column_name, ccu.table_name AS foreign_table_name, 
            ccu.column_name AS foreign_column_name 
            FROM information_schema.table_constraints AS tc 
            JOIN information_schema.key_column_usage AS kcu 
            ON tc.constraint_name = kcu.constraint_name
            JOIN information_schema.constraint_column_usage AS ccu 
            ON ccu.constraint_name = tc.constraint_name
            WHERE constraint_type = 'FOREIGN KEY'";

        sqlx::query(query)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(ForeignKey::try_from)
            .collect::<Result<Vec<ForeignKey>>>()
    }
}

impl TryFrom<PgRow> for ColumnInfo {
    type Error = anyhow::Error;

    fn try_from(row: PgRow) -> Result<Self, Self::Error> {
        Ok(ColumnInfo {
            column_name: row.try_get::<String, usize>(0)?,
            ordinal_position: row.try_get::<i32, usize>(1)?,
            is_nullable: row.try_get::<String, usize>(2)? == *"YES",
            data_type: row.try_get::<String, usize>(3)?,
            character_maximum_length: row.try_get::<Option<i32>, usize>(4)?,
        })
    }
}

impl TryFrom<PgRow> for PrimaryKey {
    type Error = anyhow::Error;

    fn try_from(row: PgRow) -> Result<Self, Self::Error> {
        Ok(PrimaryKey {
            column_name: row.try_get::<String, usize>(0)?,
            type_name: row.try_get::<String, usize>(1)?,
        })
    }
}

impl TryFrom<PgRow> for ForeignKey {
    type Error = anyhow::Error;

    fn try_from(row: PgRow) -> Result<Self, Self::Error> {
        Ok(ForeignKey {
            from_table: row.try_get::<String, usize>(0)?,
            from_column: row.try_get::<String, usize>(1)?,
            to_table: row.try_get::<String, usize>(2)?,
            to_column: row.try_get::<String, usize>(3)?
        })
    }
}