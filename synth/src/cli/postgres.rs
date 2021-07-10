use crate::cli::export::{ExportParams, ExportStrategy, create_and_insert_values};
use crate::cli::import::ImportStrategy;
use anyhow::Result;
use serde_json::Value;
use synth_core::graph::prelude::{Uuid, VariantContent};
use synth_core::schema::number_content::*;
use synth_core::schema::{
    ArrayContent, BoolContent, ChronoValueType, DateTimeContent, FieldContent, Namespace,
    NumberContent, ObjectContent, OneOfContent, RangeStep, RegexContent, StringContent,
};
use synth_core::{Content, Name};
use crate::datasource::postgres_datasource::PostgresDataSource;
use crate::datasource::DataSource;
use crate::datasource::relational_datasource::ColumnInfo;
use crate::cli::import_utils::{Collection, build_namespace_import};


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

        build_namespace_import(&datasource)
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
