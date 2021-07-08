use sqlx::{Pool, MySql, Row};
use anyhow::Result;
use crate::datasource::DataSource;
use async_std::task;
use sqlx::mysql::{MySqlPoolOptions, MySqlQueryResult, MySqlRow};
use serde_json::Value;
use async_trait::async_trait;
use crate::datasource::relational_datasource::{RelationalDataSource, ColumnInfo, PrimaryKey, ForeignKey};
use std::prelude::rust_2015::Result::Ok;
use std::convert::TryFrom;

pub struct MySqlDataSource {
    pool: Pool<MySql>,
    connect_params: String
}

#[async_trait]
impl DataSource for MySqlDataSource {
    type ConnectParams = String;

    fn new(connect_params: &Self::ConnectParams) -> Result<Self> {
        task::block_on(async {
            let pool = MySqlPoolOptions::new()
                .max_connections(3) //TODO expose this as a user config?
                .connect(connect_params.as_str())
                .await?;

            Ok::<Self, anyhow::Error>(MySqlDataSource {
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
impl RelationalDataSource for MySqlDataSource {
    type QueryResult = MySqlQueryResult;

    async fn execute_query(&self, query: String, _query_params: Vec<&str>) -> Result<MySqlQueryResult> {
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
        let query = r"SELECT table_name FROM information_schema.tables
        WHERE table_schema = 'dev' and table_type = 'BASE TABLE'";

        let table_names: Vec<String> = sqlx::query(query)
            .bind(self.get_catalog()?)
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(|row| row.get::<String, usize>(0))
            .collect();

        Ok(table_names)
    }

    async fn get_columns_infos(&self, table_name: &str) -> Result<Vec<ColumnInfo>> {
        let query = r"SELECT column_name, ordinal_position, is_nullable, data_type,
        character_maximum_length
        FROM information_schema.columns
        WHERE table_name = $1 AND table_schema = $2;";

        let column_infos = sqlx::query(query)
            .bind(table_name)
            .bind(self.get_catalog()?)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(ColumnInfo::try_from)
            .collect::<Result<Vec<ColumnInfo>>>()?;

        Ok(column_infos)
    }

    async fn get_primary_keys(&self, table_name: &str) -> Result<Vec<PrimaryKey>> {
        let query: &str = &format!(
            r"SELECT COLUMN_NAME, DATA_TYPE
            FROM INFORMATION_SCHEMA.COLUMNS
            WHERE TABLE_SCHEMA = '{}' AND TABLE_NAME = '{}' AND COLUMN_KEY = 'PRI';",
            self.get_catalog()?,
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
        let query: &str =&format!(
            r"SELECT table_name, column_name, referenced_table_name, referenced_column_name
            FROM information_schema.key_column_usage
            WHERE REFERENCED_TABLE_SCHEMA = '{}'",
            self.get_catalog()?);

        sqlx::query(query)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(ForeignKey::try_from)
            .collect::<Result<Vec<ForeignKey>>>()
    }
}

impl TryFrom<MySqlRow> for ColumnInfo {
    type Error = anyhow::Error;

    fn try_from(row: MySqlRow) -> Result<Self, Self::Error> {
        Ok(ColumnInfo {
            column_name: row.try_get::<String, usize>(0)?,
            ordinal_position: row.try_get::<i32, usize>(1)?,
            is_nullable: row.try_get::<String, usize>(2)? == *"YES",
            data_type: row.try_get::<String, usize>(3)?,
            character_maximum_length: row.try_get::<Option<i32>, usize>(4)?,
        })
    }
}

impl TryFrom<MySqlRow> for PrimaryKey {
    type Error = anyhow::Error;

    fn try_from(row: MySqlRow) -> Result<Self, Self::Error> {
        Ok(PrimaryKey {
            column_name: row.try_get::<String, usize>(0)?,
            type_name: row.try_get::<String, usize>(1)?,
        })
    }
}

impl TryFrom<MySqlRow> for ForeignKey {
    type Error = anyhow::Error;

    fn try_from(row: MySqlRow) -> Result<Self, Self::Error> {
        Ok(ForeignKey {
            from_table: row.try_get::<String, usize>(0)?,
            from_column: row.try_get::<String, usize>(1)?,
            to_table: row.try_get::<String, usize>(2)?,
            to_column: row.try_get::<String, usize>(3)?
        })
    }
}