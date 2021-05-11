use crate::cli::export::{ExportParams, ExportStrategy};
use crate::cli::import::ImportStrategy;
use anyhow::{Context, Result};

use postgres::{Client, Column, Row};
use rust_decimal::prelude::ToPrimitive;
use serde_json::{Map, Number, Value};
use std::convert::TryFrom;

use crate::sampler::Sampler;
use std::str::FromStr;
use synth_core::graph::prelude::{Uuid, VariantContent};
use synth_core::schema::number_content::*;
use synth_core::schema::{
    ArrayContent, BoolContent, ChronoValueType, DateTimeContent, FieldContent, FieldRef, Id,
    Namespace, NumberContent, ObjectContent, OneOfContent, OptionalMergeStrategy, RangeStep,
    RegexContent, SameAsContent, StringContent,
};
use synth_core::{Content, Name};

#[derive(Clone, Debug)]
pub(crate) struct PostgresExportStrategy {
    pub(crate) uri: String,
}

impl ExportStrategy for PostgresExportStrategy {
    fn export(self, params: ExportParams) -> Result<()> {
        let mut client =
            Client::connect(&self.uri, postgres::tls::NoTls).expect("Failed to connect");

        let sampler = Sampler::try_from(&params.namespace)?;
        let values = sampler.sample(params.collection_name.clone(), params.target)?;

        match values {
            Value::Array(collection_json) => {
                self.insert_data(params.collection_name.unwrap().to_string(), &collection_json, &mut client)
            }
            Value::Object(namespace_json) => {
                for (collection_name, collection_json) in namespace_json {
                    self.insert_data(
                        collection_name,
                        &collection_json
                            .as_array()
                            .expect("This is always a collection (sampler contract)"),
                        &mut client,
                    )?;
                }
                Ok(())
            }
            _ => unreachable!(
                "The sampler will never generate a value which is not an array or object (sampler contract)"
            ),
        }
    }
}

impl PostgresExportStrategy {
    fn insert_data(
        &self,
        collection_name: String,
        collection: &[Value],
        client: &mut Client,
    ) -> Result<()> {
        // how to to ordering here?
        // If we have foreign key constraints we need to traverse the tree and
        // figure out insertion order
        // We basically need something like an InsertionStrategy where we have a DAG of insertions
        let batch_size = 1000;

        let column_names = collection
            .get(0)
            .expect("Collection should not be empty")
            .as_object()
            .expect("This is always an object (sampler contract)")
            .keys()
            .cloned()
            .collect::<Vec<String>>()
            .join(",");

        for rows in collection.chunks(batch_size) {
            let mut query = format!(
                "INSERT INTO {} ({}) VALUES \n",
                collection_name, column_names
            );

            for (i, row) in rows.iter().enumerate() {
                let row_obj = row
                    .as_object()
                    .expect("This is always an object (sampler contract)");

                let values = row_obj
                    .values()
                    .map(|v| v.to_string().replace("'", "''")) // two single quotes are the standard way to escape double quotes
                    .map(|v| v.replace("\"", "'")) // values in the object have quotes around them by default.
                    .collect::<Vec<String>>()
                    .join(",");

                // We should be using some form of a prepared statement here.
                // It is not clear how this would work for our batch inserts...
                if i == rows.len() - 1 {
                    query.push_str(&format!("({});\n", values));
                } else {
                    query.push_str(&format!("({}),\n", values));
                }
            }

            if let Err(err) = client.query(query.as_str(), &[]) {
                println!("Error at: {}", query);
                return Err(err.into());
            }
        }
        println!("Inserted {} rows...", collection.len());
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PostgresImportStrategy {
    pub(crate) uri: String,
}

impl ImportStrategy for PostgresImportStrategy {
    fn import(self) -> Result<Namespace> {
        let mut client =
            Client::connect(&self.uri, postgres::tls::NoTls).expect("Failed to connect");

        let catalog = self
            .uri
            .split('/')
            .last()
            .ok_or_else(|| anyhow!("Cannot import data. No catalog specified in the uri"))?;

        let query ="SELECT table_name FROM information_schema.tables where table_catalog = $1 and table_schema = 'public' and table_type = 'BASE TABLE'";

        let table_names: Vec<String> = client
            .query(query, &[&catalog])
            .expect("Failed to get tables")
            .iter()
            .map(|row| row.get(0))
            .collect();

        let mut namespace = Namespace::default();

        // First pass - build naive Collections
        // Now we can execute a simple statement that just returns its parameter.
        for table_name in table_names.iter() {
            println!("Building {} collection...", table_name);
            // Get columns
            let col_info_query = r"select column_name, ordinal_position, is_nullable, udt_name, character_maximum_length
                 from information_schema.columns 
                 where table_name = $1 and table_catalog = $2;";

            let column_info: Vec<ColumnInfo> = client
                .query(col_info_query, &[&table_name, &catalog])
                .unwrap_or_else(|_| panic!("Failed to get columns for {}", table_name))
                .into_iter()
                .map(ColumnInfo::try_from)
                .collect::<Result<Vec<ColumnInfo>>>()?;

            namespace.put_collection(
                &Name::from_str(&table_name)?,
                Collection::from(column_info).collection,
            )?;
        }

        // Second pass - build primary keys
        println!("Building primary keys...");
        for table_name in table_names.iter() {
            // Unfortunately cannot use parameterised queries here
            let pk_query: &str = &format!(
                r"SELECT a.attname, format_type(a.atttypid, a.atttypmod) AS data_type
                    FROM   pg_index i
                    JOIN   pg_attribute a ON a.attrelid = i.indrelid
                                         AND a.attnum = ANY(i.indkey)
                    WHERE  i.indrelid = '{}'::regclass
                    AND    i.indisprimary;",
                &table_name
            );

            let primary_keys = client
                .query(pk_query, &[])
                .unwrap_or_else(|_| panic!("Failed to get primary keys for {}", table_name))
                .into_iter()
                .map(PrimaryKey::try_from)
                .collect::<Result<Vec<PrimaryKey>>>()?;

            if primary_keys.len() > 1 {
                unimplemented!("Synth does not support composite primary keys")
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
        println!("Building foreign keys...");
        let fk_query: &str = r"SELECT
                    tc.table_name, kcu.column_name,
                    ccu.table_name AS foreign_table_name,
                    ccu.column_name AS foreign_column_name
                FROM information_schema.table_constraints AS tc
                    JOIN information_schema.key_column_usage 
                        AS kcu ON tc.constraint_name = kcu.constraint_name
                    JOIN information_schema.constraint_column_usage 
                        AS ccu ON ccu.constraint_name = tc.constraint_name
                WHERE constraint_type = 'FOREIGN KEY';";

        let foreign_keys = client
            .query(fk_query, &[])
            .expect("Failed to get foreign keys")
            .into_iter()
            .map(ForeignKey::try_from)
            .collect::<Result<Vec<ForeignKey>>>()?;

        println!("{} foreign keys found.", foreign_keys.len());

        for fk in foreign_keys {
            let from_field =
                FieldRef::new(&format!("{}.content.{}", fk.from_table, fk.from_column))?;
            let to_field = FieldRef::new(&format!("{}.content.{}", fk.to_table, fk.to_column))?;
            let node = namespace.get_s_node_mut(&from_field)?;
            *node = Content::SameAs(SameAsContent { ref_: to_field });
        }

        // Fourth pass - ingest

        for table in table_names {
            print!("Ingesting data for table {}... ", table);

            // Again parameterised queries don't work here
            let data_query: &str = &format!("select * from {} order by random() limit 10;", table);
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

#[derive(Debug)]
struct ColumnInfo {
    column_name: String,
    ordinal_position: i32,
    is_nullable: bool,
    udt_name: String,
    character_maximum_length: Option<i32>,
}

#[derive(Debug)]
struct PrimaryKey {
    column_name: String,
    type_name: String,
}

#[derive(Debug)]
struct ForeignKey {
    from_table: String,
    from_column: String,
    to_table: String,
    to_column: String,
}

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

impl TryFrom<postgres::Row> for ForeignKey {
    type Error = anyhow::Error;

    fn try_from(value: Row) -> Result<Self, Self::Error> {
        Ok(ForeignKey {
            from_table: value.try_get(0)?,
            from_column: value.try_get(1)?,
            to_table: value.try_get(2)?,
            to_column: value.try_get(3)?,
        })
    }
}

impl TryFrom<postgres::Row> for PrimaryKey {
    type Error = anyhow::Error;

    fn try_from(row: Row) -> Result<Self, Self::Error> {
        Ok(PrimaryKey {
            column_name: row.try_get(0)?,
            type_name: row.try_get(1)?,
        })
    }
}

impl TryFrom<postgres::Row> for ColumnInfo {
    type Error = anyhow::Error;

    fn try_from(row: Row) -> Result<Self, Self::Error> {
        Ok(ColumnInfo {
            column_name: row.try_get(0)?,
            ordinal_position: row.try_get(1)?,
            is_nullable: row.try_get::<_, String>(2)? == *"YES",
            udt_name: row.try_get(3)?,
            character_maximum_length: row.try_get(4)?,
        })
    }
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
        let mut content = match column.udt_name.as_ref() {
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
            _ => unimplemented!("We haven't implemented a converter for {}", column.udt_name),
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
