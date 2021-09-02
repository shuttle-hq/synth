use crate::datasource::relational_datasource::{
    ColumnInfo, ForeignKey, PrimaryKey, RelationalDataSource, ValueWrapper,
};
use crate::datasource::DataSource;
use anyhow::{Context, Result};
use async_std::task;
use async_trait::async_trait;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use sqlx::postgres::{PgColumn, PgPoolOptions, PgQueryResult, PgRow};
use sqlx::{Column, Pool, Postgres, Row, TypeInfo};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use synth_core::schema::number_content::{F32, F64, I32, I64};
use synth_core::schema::{
    BoolContent, ChronoValueType, DateTimeContent, NumberContent, RangeStep, RegexContent,
    StringContent, Uuid,
};
use synth_core::{Content, Value};

pub struct PostgresDataSource {
    pool: Pool<Postgres>,
    single_thread_pool: Pool<Postgres>
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

            // Needed for queries that require explicit synchronous order, i.e. setseed + random
            let single_thread_pool = PgPoolOptions::new()
                .max_connections(1)
                .connect(connect_params.as_str())
                .await?;

            Ok::<Self, anyhow::Error>(PostgresDataSource {
                pool,
                single_thread_pool
            })
        })
    }

    async fn insert_data(&self, collection_name: String, collection: &[Value]) -> Result<()> {
        self.insert_relational_data(collection_name, collection).await
    }
}

#[async_trait]
impl RelationalDataSource for PostgresDataSource {
    type QueryResult = PgQueryResult;

    async fn execute_query(&self, query: String, query_params: Vec<&Value>) -> Result<PgQueryResult> {
        let mut query = sqlx::query(query.as_str());

        for param in query_params {
            query = query.bind(param);
        }

        let result = query.execute(&self.pool).await?;

        Ok(result)
    }

    async fn get_table_names(&self) -> Result<Vec<String>> {
        let query = r"SELECT table_name
        FROM information_schema.tables
        WHERE table_catalog = current_catalog AND table_schema = 'public' AND table_type = 'BASE TABLE'";

        sqlx::query(query)
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(|row| {
                row.try_get::<String, usize>(0)
                    .map_err(|e| anyhow!("{:?}", e))
            })
            .collect()
    }

    async fn get_columns_infos(&self, table_name: &str) -> Result<Vec<ColumnInfo>> {
        let query = r"SELECT column_name, ordinal_position, is_nullable, udt_name,
        character_maximum_length
        FROM information_schema.columns
        WHERE table_name = $1 AND table_catalog = current_catalog";

        sqlx::query(query)
            .bind(table_name)
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
        let query: &str = r"SELECT tc.table_name, kcu.column_name, ccu.table_name AS foreign_table_name, 
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

    /// Must use the singled threaded pool when setting this in conjunction with random, called by
    /// [get_deterministic_samples]. Otherwise, expect endless facepalms (-_Q)
    async fn set_seed(&self) -> Result<()> {
        sqlx::query("SELECT setseed(0.5)")
            .execute(&self.single_thread_pool)
            .await?;
        Ok(())
    }

    /// Must use the singled threaded pool when setting this in conjunction with setseed, called by
    /// [set_seed]. Otherwise, expect big regrets :(
    async fn get_deterministic_samples(&self, table_name: &str) -> Result<Vec<Value>> {
        let query: &str = &format!("SELECT * FROM {} ORDER BY random() LIMIT 10", table_name);

        let values = sqlx::query(query)
            .fetch_all(&self.single_thread_pool)
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
            "bool" => Content::Bool(BoolContent::default()),
            "oid" => {
                bail!("OID data type not supported")
            }
            "char" | "varchar" | "text" | "bpchar" | "name" | "unknown" => {
                let pattern =
                    "[a-zA-Z0-9]{0, {}}".replace("{}", &format!("{}", char_max_len.unwrap_or(1)));
                Content::String(StringContent::Pattern(
                    RegexContent::pattern(pattern).context("pattern will always compile")?,
                ))
            }
            "int2" => Content::Number(NumberContent::I64(I64::Range(RangeStep {
                low: 0,
                high: 1,
                step: 1,
            }))),
            "int4" => Content::Number(NumberContent::I32(I32::Range(RangeStep {
                low: 0,
                high: 1,
                step: 1,
            }))),
            "int8" => Content::Number(NumberContent::I64(I64::Range(RangeStep {
                low: 0,
                high: 1,
                step: 1,
            }))),
            "float4" => Content::Number(NumberContent::F32(F32::Range(RangeStep {
                low: 0.0,
                high: 1.0,
                step: 0.1, //todo
            }))),
            "float8" => Content::Number(NumberContent::F64(F64::Range(RangeStep {
                low: 0.0,
                high: 1.0,
                step: 0.1, //todo
            }))),
            "numeric" => Content::Number(NumberContent::F64(F64::Range(RangeStep {
                low: 0.0,
                high: 1.0,
                step: 0.1, //todo
            }))),
            "timestamptz" => Content::String(StringContent::DateTime(DateTimeContent {
                format: "".to_string(), // todo
                type_: ChronoValueType::DateTime,
                begin: None,
                end: None,
            })),
            "timestamp" => Content::String(StringContent::DateTime(DateTimeContent {
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
            "uuid" => Content::String(StringContent::Uuid(Uuid)),
            _ => bail!("We haven't implemented a converter for {}", data_type),
        };

        Ok(content)
    }

    fn extend_parameterised_query(query: &mut String, curr_index: usize, extend: usize) {
        query.push('(');
        for i in 0..extend {
            query.push_str(&format!("${}", curr_index + i + 1));
            if i != extend - 1 {
                query.push(',');
            }
        }
        query.push(')');
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
            to_column: row.try_get::<String, usize>(3)?,
        })
    }
}

impl TryFrom<PgRow> for ValueWrapper {
    type Error = anyhow::Error;

    fn try_from(row: PgRow) -> Result<Self, Self::Error> {
        let mut kv = BTreeMap::new();

        for column in row.columns() {
            let value = try_match_value(&row, column).unwrap_or(Value::Null(()));
            kv.insert(column.name().to_string(), value);
        }

        Ok(ValueWrapper(Value::Object(kv)))
    }
}

fn try_match_value(row: &PgRow, column: &PgColumn) -> Result<Value> {
    let value = match column.type_info().name().to_lowercase().as_str() {
        "bool" => Value::Bool(row.try_get::<bool, &str>(column.name())?),
        "oid" => {
            bail!("OID data type not supported for Postgresql")
        }
        "char" | "varchar" | "text" | "bpchar" | "name" | "unknown" => {
            Value::String(row.try_get::<String, &str>(column.name())?)
        }
        "int2" => Value::Number(row.try_get::<i16, &str>(column.name())?.into()),
        "int4" => Value::Number(row.try_get::<i32, &str>(column.name())?.into()),
        "int8" => Value::Number(row.try_get::<i64, &str>(column.name())?.into()),
        "float4" => Value::Number(row.try_get::<f32, &str>(column.name())?.into()),
        "float8" => Value::Number(row.try_get::<f64, &str>(column.name())?.into()),
        "numeric" => {
            let as_decimal = row.try_get::<Decimal, &str>(column.name())?;

            if let Some(truncated) = as_decimal.to_f64() {
                return Ok(Value::Number(truncated.into()));
            }

            bail!("Failed to convert Postgresql numeric data type to 64 bit float")
        }
        "timestampz" => Value::String(row.try_get::<String, &str>(column.name())?),
        "timestamp" => Value::String(row.try_get::<String, &str>(column.name())?),
        "date" => Value::String(format!(
            "{}",
            row.try_get::<chrono::NaiveDate, &str>(column.name())?
        )),
        _ => {
            bail!(
                "Could not convert value. Converter not implemented for {}",
                column.type_info().name()
            );
        }
    };

    Ok(value)
}
