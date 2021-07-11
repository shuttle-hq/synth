use sqlx::{Pool, MySql, Row, Column, TypeInfo};
use anyhow::{Result, Context};
use crate::datasource::DataSource;
use async_std::task;
use sqlx::mysql::{MySqlPoolOptions, MySqlQueryResult, MySqlRow, MySqlColumn};
use serde_json::{Value, Map, Number};
use async_trait::async_trait;
use crate::datasource::relational_datasource::{RelationalDataSource, ColumnInfo, PrimaryKey, ForeignKey, ValueWrapper};
use std::prelude::rust_2015::Result::Ok;
use std::convert::TryFrom;
use synth_core::Content;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use synth_core::schema::{BoolContent, StringContent, RegexContent, NumberContent, RangeStep, DateTimeContent, ChronoValueType};
use synth_core::schema::number_content::{I64, F64, U64};

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
        let query = &format!(r"SELECT column_name, ordinal_position, is_nullable, data_type,
        character_maximum_length
        FROM information_schema.columns
        WHERE table_name = '{}' AND table_schema = '{}'", table_name, self.get_catalog()?);

        let column_infos = sqlx::query(query)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(ColumnInfo::try_from)
            .collect::<Result<Vec<ColumnInfo>>>()?;

        Ok(column_infos)
    }

    async fn get_primary_keys(&self, table_name: &str) -> Result<Vec<PrimaryKey>> {
        let query: &str = &format!(
            r"SELECT column_name, data_type
            FROM information_schema.columns
            WHERE table_schema = '{}' AND table_name = '{}' AND column_key = 'PRI'",
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
            WHERE referenced_table_schema = '{}'",
            self.get_catalog()?);

        sqlx::query(query)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(ForeignKey::try_from)
            .collect::<Result<Vec<ForeignKey>>>()
    }

    async fn get_deterministic_samples(&self, table_name: &str) -> Result<Vec<Value>> {
        let query: &str = &format!("SELECT * FROM {} ORDER BY rand(0.5) LIMIT 10", table_name);

        let values = sqlx::query(query)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(ValueWrapper::try_from)
            .map(|v| {
                match v {
                    Ok(wrapper) => Ok(wrapper.0),
                    Err(e) => bail!("Failed to convert to value wrapper from query results: {:?}", e)
                }
            })
            .collect::<Result<Vec<Value>>>()?;

        Ok(values)
    }

    fn decode_to_content(&self, data_type: &str, char_max_len: Option<i32>) -> Result<Content> {
        let content = match data_type {
            "bool" | "boolean" => Content::Bool(BoolContent::default()),
            "char" | "varchar" | "text" | "binary" | "varbinary" | "enum" | "set" => {
                let pattern = "[a-zA-Z0-9]{0, {}}".replace(
                    "{}",
                    &format!("{}", char_max_len.unwrap_or(1)),
                );
                Content::String(StringContent::Pattern(
                    RegexContent::pattern(pattern).context("pattern will always compile")?,
                ))
            },
            "int" | "integer" | "tinyint" | "smallint" | "mediumint" | "bigint" =>
                Content::Number(NumberContent::I64(I64::Range(RangeStep {
                    low: 0,
                    high: 1,
                    step: 1,
                }))),
            "serial" => Content::Number(NumberContent::U64(U64::Range(RangeStep {
                low: 0,
                high: 1,
                step: 1,
            }))),
            name if name.starts_with("float") |
                name.starts_with("double") |
                name.starts_with("numeric") |
                name.starts_with("decimal") =>
                Content::Number(NumberContent::F64(F64::Range(RangeStep {
                low: 0.0,
                high: 1.0,
                step: 0.1, //todo
            }))),
            name if name.starts_with("timestamp") => Content::String(StringContent::DateTime(DateTimeContent {
                format: "".to_string(), // todo
                type_: ChronoValueType::NaiveDateTime,
                begin: None,
                end: None,
            })),
            "date" => Content::String(StringContent::DateTime(DateTimeContent {
                format: "%Y-%m-%d".to_string(),
                type_: ChronoValueType::NaiveDate,
                begin: None,
                end: None,
            })),
            //TODO time datetime
            // name if name.starts_with("datetime") => Content::String(StringContent::DateTime(DateTimeContent {
            //     format: "%Y-%m-%d".to_string(),
            //     type_: ChronoValueType::NaiveDateTime,
            //     begin: None,
            //     end: None,
            // })),
            _ => bail!("We haven't implemented a converter for {}", data_type),
        };

        Ok(content)
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

impl TryFrom<MySqlRow> for ValueWrapper {
    type Error = anyhow::Error;

    fn try_from(row: MySqlRow) -> Result<Self, Self::Error> {
        let mut json_kv = Map::new();

        for column in row.columns() {
            let value = try_match_value(&row, column).unwrap_or(Value::Null);
            json_kv.insert(column.name().to_string(), value);
        }

        Ok(ValueWrapper(Value::Object(json_kv)))
    }
}

fn try_match_value(row: &MySqlRow, column: &MySqlColumn) -> Result<Value> {
    let value = match column.type_info().name() {
        "bool" | "boolean" => Value::Bool(row.try_get::<bool, &str>(column.name())?),
        "char" | "varchar" | "text" | "binary" | "varbinary" | "enum" | "set" => {
            Value::String(row.try_get::<String, &str>(column.name())?)
        }
        "tinyint" => Value::Number(Number::from(row.try_get::<i8, &str>(column.name())?)),
        "smallint" => Value::Number(Number::from(row.try_get::<i16, &str>(column.name())?)),
        "mediumint" | "int" | "integer" => Value::Number(Number::from(row.try_get::<i32, &str>(column.name())?)),
        "bigint" => Value::Number(Number::from(row.try_get::<i64, &str>(column.name())?)),
        "serial" => Value::Number(Number::from(row.try_get::<u64, &str>(column.name())?)),
        name if name.starts_with("float") => Value::Number(Number::from_f64(row.try_get::<f32, &str>(column.name())? as f64)
            .ok_or(anyhow!("Failed to convert float data type"))?), // TODO test f32, f64
        name if name.starts_with("double") => Value::Number(Number::from_f64(row.try_get::<f64, &str>(column.name())?)
            .ok_or(anyhow!("Failed to convert double data type"))?),
        name if name.starts_with("numeric") | name.starts_with("decimal") => {
            let as_decimal = row.try_get::<Decimal, &str>(column.name())?;

            if let Some(truncated) = as_decimal.to_f64() {
                if let Some(json_number) =  Number::from_f64(truncated) {
                    return Ok(Value::Number(json_number));
                }
            }

            bail!("Failed to convert Postgresql numeric data type to 64 bit float")
        }
        name if name.starts_with("timestamp") => Value::String(
            row.try_get::<String, &str>(column.name())?),
        "date" => Value::String(format!("{}", row.try_get::<chrono::NaiveDate, &str>(column.name())?)),
        name if name.starts_with("datetime") => Value::String(
            format!("{}", row.try_get::<chrono::NaiveDateTime, &str>(column.name())?)),
        name if name.starts_with("time") => Value::String(
            format!("{}", row.try_get::<chrono::NaiveTime, &str>(column.name())?)),
        _ => {
            bail!(
                "Could not convert value. Converter not implemented for {}",
                column.type_info().name()
            );
        }
    };

    Ok(value)
}