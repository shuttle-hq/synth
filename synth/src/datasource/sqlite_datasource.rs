use crate::datasource::relational_datasource::{
    ColumnInfo, ForeignKey, PrimaryKey, RelationalDataSource, ValueWrapper,
};
use crate::datasource::DataSource;
use anyhow::{Context, Result};
use async_std::task;
use async_trait::async_trait;
use rust_decimal::prelude::ToPrimitive;
use sqlx::sqlite::{SqliteColumn, SqlitePoolOptions, SqliteQueryResult, SqliteRow};
use sqlx::{Column, Pool, Row, Sqlite, TypeInfo};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::prelude::rust_2015::Result::Ok;
use synth_core::schema::number_content::{F64, I64};
use synth_core::schema::{
    BoolContent, ChronoValueType, DateTimeContent, NullContent, NumberContent, RangeStep,
    RegexContent, StringContent,
};
use synth_core::{Content, Value};
use synth_gen::prelude::*;

/// TODO
/// Known issues:
/// - Sqlite's random implementation does not support a seed argument. We currently use `random` directly.
/// This makes the sampling not behave as intended.

pub struct SqliteDataSource {
    pool: Pool<Sqlite>,
}

#[async_trait]
impl DataSource for SqliteDataSource {
    type ConnectParams = String;

    fn new(connect_params: &Self::ConnectParams) -> Result<Self> {
        task::block_on(async {
            use sqlx::migrate::MigrateDatabase;
            if !sqlx::Sqlite::database_exists(connect_params.as_str()).await? {
                sqlx::Sqlite::create_database(connect_params.as_str()).await?;
            }

            let pool = SqlitePoolOptions::new()
                .max_connections(3) //TODO expose this as a user config?
                .connect(connect_params.as_str())
                .await?;

            Ok::<Self, anyhow::Error>(SqliteDataSource { pool })
        })
    }

    async fn insert_data(&self, collection_name: String, collection: &[Value]) -> Result<()> {
        self.insert_relational_data(collection_name, collection)
            .await
    }
}

#[async_trait]
impl RelationalDataSource for SqliteDataSource {
    type QueryResult = SqliteQueryResult;

    async fn execute_query(
        &self,
        query: String,
        query_params: Vec<&Value>,
    ) -> Result<SqliteQueryResult> {
        let mut query = sqlx::query(query.as_str());

        for param in query_params {
            query = query.bind(param);
        }

        let result = query.execute(&self.pool).await?;

        Ok(result)
    }

    async fn get_table_names(&self) -> Result<Vec<String>> {
        let query = r"SELECT name FROM sqlite_master
            WHERE type='table'";

        let table_names: Vec<String> = sqlx::query(query)
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(|row| row.get::<String, usize>(0))
            .collect();

        Ok(table_names)
    }

    async fn get_columns_infos(&self, table_name: &str) -> Result<Vec<ColumnInfo>> {
        let query = r"SELECT * FROM PRAGMA_TABLE_INFO(?)";

        let column_infos = sqlx::query(query)
            .bind(table_name)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(ColumnInfo::try_from)
            .collect::<Result<Vec<ColumnInfo>>>()?;

        Ok(column_infos)
    }

    async fn get_primary_keys(&self, table_name: &str) -> Result<Vec<PrimaryKey>> {
        let query: &str = r"SELECT name, type FROM pragma_table_info(?) WHERE pk = 1";

        sqlx::query(query)
            .bind(table_name)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(PrimaryKey::try_from)
            .collect::<Result<Vec<PrimaryKey>>>()
    }

    async fn get_foreign_keys(&self) -> Result<Vec<ForeignKey>> {
        let query: &str = r"SELECT * FROM pragma_table_info(?)";

        sqlx::query(query)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(ForeignKey::try_from)
            .collect::<Result<Vec<ForeignKey>>>()
    }

    async fn set_seed(&self) -> Result<()> {
        // Sqlite doesn't set seed in a separate query
        Ok(())
    }

    async fn get_deterministic_samples(&self, table_name: &str) -> Result<Vec<Value>> {
        // FIXME:(rasviitanen) [2021-10-03] The random implementation doesn't take a seed
        // in Sqlite, should we use rust's rand instead?
        let query: &str = &format!("SELECT * FROM {} ORDER BY random() LIMIT 10", table_name);

        let values = sqlx::query(query)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(ValueWrapper::try_from)
            .map(|v| match v {
                Ok(wrapper) => Ok(wrapper.0),
                Err(e) => bail!(
                    "Failed to convert to value wrapper from query results: {:?}",
                    e
                ),
            })
            .collect::<Result<Vec<Value>>>()?;

        Ok(values)
    }

    fn decode_to_content(&self, data_type: &str, char_max_len: Option<i32>) -> Result<Content> {
        let content = match data_type.to_lowercase().as_str() {
            "boolean" => {
                Content::Bool(BoolContent::default())
            }
            "integer" => {
                Content::Number(NumberContent::I64(I64::Range(RangeStep::default())))
            }
            "int8" | "bigint" => {
            // FIXME:(rasviitanen) [2021-10-03] this should be i128, but is fine for now as u64 is not supported yet
                Content::Number(NumberContent::I64(I64::Range(RangeStep::default())))
            }
            "real" => {
                Content::Number(NumberContent::F64(F64::Range(RangeStep::default())))
            }
            "datetime" => {
                Content::String(StringContent::DateTime(DateTimeContent {
                    format: "%Y-%m-%d %H:%M:%S".to_string(),
                    type_: ChronoValueType::NaiveDateTime,
                    begin: None,
                    end: None,
                }))
            }
            "blob" | "text" => {
                let pattern =
                    "[a-zA-Z0-9]{0, {}}".replace("{}", &format!("{}", char_max_len.unwrap_or(1)));
                Content::String(StringContent::Pattern(
                    RegexContent::pattern(pattern).context("pattern will always compile")?,
                ))
            }
            "null" => Content::Null(NullContent),
            _ => unreachable!(
                "Missing converter implementation for `{}`, but synth's Sqlite decoder should cover all types. \
                Please reach out to https://github.com/getsynth/synth/issues if encountered.",
                data_type
            ),
        };

        Ok(content)
    }

    fn extend_parameterised_query(query: &mut String, _curr_index: usize, extend: usize) {
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

impl TryFrom<SqliteRow> for ColumnInfo {
    type Error = anyhow::Error;

    fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
        Ok(ColumnInfo {
            column_name: row.try_get::<String, usize>(1)?,
            ordinal_position: row.try_get::<u32, usize>(0)? as i32,
            is_nullable: !row.try_get::<bool, usize>(3)?,
            data_type: row.try_get::<String, usize>(2)?,
            character_maximum_length: None,
        })
    }
}

impl TryFrom<SqliteRow> for PrimaryKey {
    type Error = anyhow::Error;

    fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
        Ok(PrimaryKey {
            column_name: row.try_get::<String, usize>(0)?,
            type_name: row.try_get::<String, usize>(1)?,
        })
    }
}

impl TryFrom<SqliteRow> for ForeignKey {
    type Error = anyhow::Error;

    fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
        Ok(ForeignKey {
            from_table: row.try_get::<String, usize>(0)?,
            from_column: row.try_get::<String, usize>(1)?,
            to_table: row.try_get::<String, usize>(2)?,
            to_column: row.try_get::<String, usize>(3)?,
        })
    }
}

impl TryFrom<SqliteRow> for ValueWrapper {
    type Error = anyhow::Error;

    fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
        let mut kv = BTreeMap::new();

        for column in row.columns() {
            let value = try_match_value(&row, column).unwrap_or(Value::Null(()));
            kv.insert(column.name().to_string(), value);
        }

        Ok(ValueWrapper(Value::Object(kv)))
    }
}

fn try_match_value(row: &SqliteRow, column: &SqliteColumn) -> Result<Value> {
    let value = match column.type_info().name().to_lowercase().as_str() {
        "boolean" => {
            Value::Bool(row.try_get::<i8, &str>(column.name())? == 1)
        }
        "integer" => {
            Value::Number(Number::from(row.try_get::<i64, &str>(column.name())?))
        }
        "int8" | "bigint" => {
            // FIXME:(rasviitanen) [2021-10-03] this should be i128, but is fine for now as u64 is not supported yet
            Value::Number(Number::from(row.try_get::<i64, &str>(column.name())?))
        }
        "real" => {
            let as_decimal = row.try_get::<u32, &str>(column.name())?;

            if let Some(truncated) = as_decimal.to_f64() {
                return Ok(Value::Number(Number::from(truncated)));
            }

            bail!("Failed to convert Sqlite real data type to 64 bit float")
        }
        "datetime" => Value::String(format!(
            "{}",
            row.try_get::<chrono::NaiveDateTime, &str>(column.name())?
        )),
        "blob" | "text" => {
            Value::String(row.try_get::<String, &str>(column.name())?)
        }
        "null" => Value::Null(()),
        _ => {
            bail!(
                "Could not convert value. Converter not implemented for {}, but Synth's Sqlite converter should cover all types. \
                Please reach out to https://github.com/getsynth/synth/issues if encountered.",
                column.type_info().name()
            );
        }
    };

    Ok(value)
}
