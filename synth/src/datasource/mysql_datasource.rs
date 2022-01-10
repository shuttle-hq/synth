use crate::datasource::relational_datasource::{
    insert_relational_data, ColumnInfo, ForeignKey, PrimaryKey, SqlxDataSource,
};
use crate::datasource::DataSource;
use anyhow::{Context, Result};
use async_std::task;
use async_trait::async_trait;
use sqlx::mysql::{MySqlPoolOptions, MySqlRow};
use sqlx::{MySql, Pool, Row};
use std::convert::TryFrom;
use std::prelude::rust_2015::Result::Ok;
use synth_core::schema::number_content::{F64, I64, U64};
use synth_core::schema::{
    ChronoValueType, DateTimeContent, NumberContent, RangeStep, RegexContent, StringContent,
};
use synth_core::{Content, Value};

/// TODO
/// Known issues:
/// - MySql aliases bool and boolean data types as tinyint. We currently define all tinyint as i8.
///   Ideally, the user can define a way to force certain fields as bool rather than i8.

pub struct MySqlDataSource {
    pool: Pool<MySql>,
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

            Ok::<Self, anyhow::Error>(MySqlDataSource { pool })
        })
    }

    async fn insert_data(&self, collection_name: &str, collection: &[Value]) -> Result<()> {
        insert_relational_data(self, collection_name, collection).await
    }
}

impl SqlxDataSource for MySqlDataSource {
    type DB = MySql;
    type Arguments = sqlx::mysql::MySqlArguments;
    type Connection = sqlx::mysql::MySqlConnection;

    const IDENTIFIER_QUOTE: char = '`';

    fn get_pool(&self) -> Pool<Self::DB> {
        Pool::clone(&self.pool)
    }

    fn get_multithread_pool(&self) -> Pool<Self::DB> {
        Pool::clone(&self.pool)
    }

    fn get_table_names_query(&self) -> &str {
        r"SELECT table_name FROM information_schema.tables
            WHERE table_schema = DATABASE() and table_type = 'BASE TABLE'"
    }

    fn get_primary_keys_query(&self) -> &str {
        r"SELECT column_name, data_type
            FROM information_schema.columns
            WHERE table_schema = DATABASE() AND table_name = ? AND column_key = 'PRI'"
    }

    fn get_foreign_keys_query(&self) -> &str {
        r"SELECT table_name, column_name, referenced_table_name, referenced_column_name
            FROM information_schema.key_column_usage
            WHERE referenced_table_schema = DATABASE()"
    }

    fn get_deterministic_samples_query(&self, table_name: String) -> String {
        format!("SELECT * FROM {} ORDER BY rand(0.5) LIMIT 10", table_name)
    }

    fn decode_to_content(&self, column_info: &ColumnInfo) -> Result<Content> {
        let content = match column_info.data_type.to_lowercase().as_str() {
            "char" | "varchar" | "text" | "binary" | "varbinary" | "enum" | "set" => {
                let pattern = "[a-zA-Z0-9]{0, {}}".replace(
                    "{}",
                    &format!("{}", column_info.character_maximum_length.unwrap_or(1)),
                );
                Content::String(StringContent::Pattern(
                    RegexContent::pattern(pattern).context("pattern will always compile")?,
                ))
            }
            "int" | "integer" | "tinyint" | "smallint" | "mediumint" | "bigint" => {
                Content::Number(NumberContent::I64(I64::Range(RangeStep::default())))
            }
            "serial" => Content::Number(NumberContent::U64(U64::Range(RangeStep::default()))),
            "float" | "double" | "numeric" | "decimal" => {
                Content::Number(NumberContent::F64(F64::Range(RangeStep::default())))
            }
            "timestamp" => Content::DateTime(DateTimeContent {
                format: "".to_string(), // todo
                type_: ChronoValueType::NaiveDateTime,
                begin: None,
                end: None,
            }),
            "date" => Content::DateTime(DateTimeContent {
                format: "%Y-%m-%d".to_string(),
                type_: ChronoValueType::NaiveDate,
                begin: None,
                end: None,
            }),
            "datetime" => Content::DateTime(DateTimeContent {
                format: "%Y-%m-%d %H:%M:%S".to_string(),
                type_: ChronoValueType::NaiveDateTime,
                begin: None,
                end: None,
            }),
            "time" => Content::DateTime(DateTimeContent {
                format: "%H:%M:%S".to_string(),
                type_: ChronoValueType::NaiveTime,
                begin: None,
                end: None,
            }),
            _ => bail!(
                "We haven't implemented a converter for {}",
                column_info.data_type
            ),
        };

        Ok(content)
    }

    fn get_columns_info_query(&self) -> &str {
        r"SELECT column_name, ordinal_position, is_nullable, data_type,
            character_maximum_length
            FROM information_schema.columns
            WHERE table_name = ? AND table_schema = DATABASE()"
    }
}

impl TryFrom<MySqlRow> for ColumnInfo {
    type Error = anyhow::Error;

    fn try_from(row: MySqlRow) -> Result<Self, Self::Error> {
        Ok(ColumnInfo {
            column_name: row.try_get::<String, usize>(0)?,
            ordinal_position: row.try_get::<u32, usize>(1)? as i32,
            is_nullable: row.try_get::<String, usize>(2)? == *"YES",
            data_type: row.try_get::<String, usize>(3)?,
            character_maximum_length: extract_column_char_max_len(4, row)?,
            is_custom_type: false,
        })
    }
}

/// Extracts a column's max character length. MySql's datatype for max char length is INT, but for
/// Mariadb it's BIGINT UNSIGNED, so we must try both rust data types when reading the row. We
/// truncate i64 to i32 in order to fit our internal models and practically, we probably won't be
/// generating synthetic data for sizes beyond i32.
fn extract_column_char_max_len(index: usize, row: MySqlRow) -> Result<Option<i32>> {
    let character_maximum_length = match row.try_get(index) {
        Ok(c) => c,
        Err(_) => row.try_get::<Option<u64>, usize>(index)?.map(|c| c as i32),
    };

    Ok(character_maximum_length)
}

impl TryFrom<MySqlRow> for PrimaryKey {
    type Error = anyhow::Error;

    fn try_from(row: MySqlRow) -> Result<Self, Self::Error> {
        Ok(PrimaryKey {
            column_name: row.try_get(0)?,
            type_name: row.try_get(1)?,
        })
    }
}

impl TryFrom<MySqlRow> for ForeignKey {
    type Error = anyhow::Error;

    fn try_from(row: MySqlRow) -> Result<Self, Self::Error> {
        Ok(ForeignKey {
            from_table: row.try_get(0)?,
            from_column: row.try_get(1)?,
            to_table: row.try_get(2)?,
            to_column: row.try_get(3)?,
        })
    }
}
