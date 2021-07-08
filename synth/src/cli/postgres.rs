use crate::cli::export::{ExportParams, ExportStrategy, create_and_insert_values};
use crate::cli::import::ImportStrategy;
use anyhow::{Context, Result};
use async_std::task;

use postgres::{Client, Column, Row};
use rust_decimal::prelude::ToPrimitive;
use serde_json::{Map, Number, Value};
use std::convert::TryFrom;
use std::str::FromStr;
use synth_core::graph::prelude::{Uuid, VariantContent};
use synth_core::schema::number_content::*;
use synth_core::schema::{
    ArrayContent, BoolContent, ChronoValueType, DateTimeContent, FieldContent, FieldRef, Id,
    Namespace, NumberContent, ObjectContent, OneOfContent, OptionalMergeStrategy, RangeStep,
    RegexContent, SameAsContent, StringContent,
};
use synth_core::{Content, Name};
use crate::datasource::postgres_datasource::PostgresDataSource;
use crate::datasource::DataSource;
use crate::datasource::relational_datasource::{RelationalDataSource, ColumnInfo, PrimaryKey, ForeignKey};


#[derive(Clone, Debug)]
pub struct PostgresExportStrategy {
    pub uri: String,
}

impl ExportStrategy for PostgresExportStrategy {
    fn export(self, params: ExportParams) -> Result<()> {
        let datasource = PostgresDataSource::new(&self.uri)?;

        create_and_insert_values(params, &datasource)
    }
}

#[derive(Clone, Debug)]
pub struct PostgresImportStrategy {
    pub uri: String,
}

impl ImportStrategy for PostgresImportStrategy {
    fn import(self) -> Result<Namespace> {
        let datasource = PostgresDataSource::new(&self.uri)?;

        let table_names = task::block_on(async {
            let table_names = datasource.get_table_names().await?;
            Ok::<Vec<String>, anyhow::Error>(table_names)
        }).context(format!("Failed to get table names"))?;

        let mut namespace = Namespace::default();

        // First pass - build naive Collections
        // Now we can execute a simple statement that just returns its parameter.
        for table_name in table_names.iter() {
            println!("Building {} collection...", table_name);

            let column_infos = task::block_on(async {
                Ok::<Vec<ColumnInfo>, anyhow::Error>(datasource.get_columns_infos(table_name).await?)
            })?;

            namespace.put_collection(
                &Name::from_str(&table_name)?,
                Collection::from(column_infos).collection,
            )?;
        }

        // Second pass - build primary keys
        println!("Building primary keys...");
        for table_name in table_names.iter() {
            let primary_keys = task::block_on(async {
                Ok::<Vec<PrimaryKey>, anyhow::Error>(datasource.get_primary_keys(table_name).await?)
            })?;

            if primary_keys.len() > 1 {
                bail!("Synth does not support composite primary keys")
            }

            if let Some(primary_key) = primary_keys.get(0) {
                let field = FieldRef::new(&format!(
                    "{}.content.{}",
                    table_name, primary_key.column_name
                ))?;
                let node = namespace.get_s_node_mut(&field)?;
                *node = Content::Number(NumberContent::U64(U64::Id(Id::default())));
            }
        }

        // Third pass - foreign keys
        let foreign_keys = task::block_on(async {
            Ok::<Vec<ForeignKey>, anyhow::Error>(datasource.get_foreign_keys().await?)
        })?;

        println!("{} foreign keys found.", foreign_keys.len());

        for fk in foreign_keys {
            let from_field =
                FieldRef::new(&format!("{}.content.{}", fk.from_table, fk.from_column))?;
            let to_field = FieldRef::new(&format!("{}.content.{}", fk.to_table, fk.to_column))?;
            let node = namespace.get_s_node_mut(&from_field)?;
            *node = Content::SameAs(SameAsContent { ref_: to_field });
        }






        //TODO remove this
        let mut client =
            Client::connect(&self.uri, postgres::tls::NoTls).expect("Failed to connect");










        // Set the RNG to get a deterministic sample
        let _ = client
            .query("select setseed(0.5);", &[])
            .expect("Failed to set seed for sampling");

        // Fourth pass - ingest
        for table in table_names {
            println!("Ingesting data for table {}... ", table);

            // Again parameterised queries don't work here
            let data_query: &str = &format!("select * from {} order by random() limit 10;", table);

            println!("data_query = {}", data_query);

            let data = client
                .query(data_query, &[])
                .expect("Failed to retrieve data");
            println!(" {} rows done.", data.len());

            let values: Values = Values::try_from(data).context(format!("at table {}", table))?;

            namespace.try_update(
                OptionalMergeStrategy,
                &Name::from_str(&table).unwrap(),
                &Value::from(values.0),
            )?;
        }

        Ok(namespace)
    }

    fn import_collection(self, name: &Name) -> Result<Content> {
        self.import()?
            .collections
            .remove(name)
            .ok_or_else(|| anyhow!("Could not find table '{}' in Postgres database.", name))
    }

    fn into_value(self) -> Result<Value> {
        unreachable!()
    }
}

// Wrapper around content
#[derive(Debug)]
struct Collection {
    collection: Content,
}

// Wrapper around rows
#[derive(Debug)]
struct Values(Vec<Value>);

impl TryFrom<Vec<Row>> for Values {
    type Error = anyhow::Error;

    // Need to check for nulls here.
    // Not sure how we're going to do this. We need some macro to try_get, and if it fails
    // go for content::null
    // https://kotiri.com/2018/01/31/postgresql-diesel-rust-types.html
    fn try_from(rows: Vec<Row>) -> Result<Self, Self::Error> {
        let mut values = Vec::new();
        for row in rows {
            let mut obj_content = Map::new();
            for i in 0..row.columns().len() {
                let column = row
                    .columns()
                    .get(i)
                    .expect("Cannot go out of range here since iterator is bound by length");
                let value = try_match_value(&row, i, column).unwrap_or(Value::Null);
                obj_content.insert(column.name().to_string(), value);
            }
            values.push(Value::Object(obj_content));
        }
        Ok(Values(values))
    }
}

fn try_match_value(row: &Row, i: usize, column: &Column) -> Result<Value> {
    let value = match column.type_().name() {
        "bool" => Value::Bool(row.try_get(i)?),
        "oid" => {
            unimplemented!()
        }
        "char" | "varchar" | "text" | "bpchar" | "name" | "unknown" => {
            Value::String(row.try_get(i)?)
        }
        "int2" => Value::Number(Number::from(row.try_get::<_, i16>(i)?)),
        "int4" => Value::Number(Number::from(row.try_get::<_, i32>(i)?)),
        "int8" => Value::Number(Number::from(row.try_get::<_, i64>(i)?)),
        "float4" => Value::Number(
            Number::from_f64(row.try_get(i)?).expect("Cloud not convert to f64. Value was NaN."),
        ),
        "float8" => Value::Number(
            Number::from_f64(row.try_get(i)?).expect("Cloud not convert to f64. Value was NaN."),
        ),
        "numeric" => {
            let as_decimal: rust_decimal::Decimal = row.try_get(i)?;
            Value::Number(
                Number::from_f64(
                    as_decimal
                        .to_f64()
                        .expect("Could not convert decimal to f64 for reasons todo"),
                )
                .expect("Cloud not convert to f64. Value was NaN."),
            )
        }
        "timestampz" => Value::String(row.try_get(i)?),
        "timestamp" => Value::String(row.try_get(i)?),
        "date" => Value::String(format!("{}", row.try_get::<_, chrono::NaiveDate>(i)?)),
        _ => {
            return Err(anyhow!(
                "Could not convert value. Converter not implemented for {}",
                column.type_().name()
            ));
        }
    };
    Ok(value)
}

impl From<Vec<ColumnInfo>> for Collection {
    fn from(columns: Vec<ColumnInfo>) -> Self {
        let mut collection = ObjectContent::default();

        for column in columns {
            collection
                .fields
                .insert(column.column_name.clone(), column.into());
        }

        Collection {
            collection: Content::Array(ArrayContent {
                length: Box::new(Content::Number(NumberContent::U64(U64::Range(RangeStep {
                    low: 1,
                    high: 2,
                    step: 1,
                })))),
                content: Box::new(Content::Object(collection)),
            }),
        }
    }
}

// Type conversions: https://docs.rs/postgres-types/0.2.0/src/postgres_types/lib.rs.html#360
impl From<ColumnInfo> for FieldContent {
    fn from(column: ColumnInfo) -> Self {
        let mut content = match column.data_type.as_ref() {
            "bool" => Content::Bool(BoolContent::default()),
            "oid" => {
                unimplemented!()
            }
            "char" | "varchar" | "text" | "bpchar" | "name" | "unknown" => {
                let pattern = "[a-zA-Z0-9]{0, {}}".replace(
                    "{}",
                    &format!("{}", column.character_maximum_length.unwrap_or(1)),
                );
                Content::String(StringContent::Pattern(
                    RegexContent::pattern(pattern).expect("pattern will always compile"),
                ))
            }
            "int2" => Content::Number(NumberContent::I64(I64::Range(RangeStep {
                low: 0,
                high: 1,
                step: 1,
            }))),
            "int4" => Content::Number(NumberContent::I64(I64::Range(RangeStep {
                low: 0,
                high: 1,
                step: 1,
            }))),
            "int8" => Content::Number(NumberContent::I64(I64::Range(RangeStep {
                low: 1,
                high: 1,
                step: 1,
            }))),
            "float4" => Content::Number(NumberContent::F64(F64::Range(RangeStep {
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
            "timestampz" => Content::String(StringContent::DateTime(DateTimeContent {
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
            _ => unimplemented!("We haven't implemented a converter for {}", column.data_type),
        };

        // This happens because an `optional` field in a Synth schema
        // won't show up as a key during generation. Whereas what we
        // want instead is a null field.
        if column.is_nullable {
            content = Content::OneOf(OneOfContent {
                variants: vec![
                    VariantContent::new(content),
                    VariantContent::new(Content::Null),
                ],
            })
        }

        FieldContent {
            optional: false,
            content: Box::new(content),
        }
    }
}
