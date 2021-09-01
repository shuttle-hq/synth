use crate::datasource::relational_datasource::{ColumnInfo, RelationalDataSource};
use crate::datasource::DataSource;
use anyhow::{Context, Result};
use async_std::task;
use log::debug;
use serde_json::Value;
use std::convert::TryFrom;
use std::str::FromStr;
use synth_core::schema::content::number_content::U64;
use synth_core::schema::{
    ArrayContent, FieldRef, Id, NumberContent, ObjectContent, OptionalMergeStrategy, RangeStep,
    SameAsContent, StringContent, FakerContent
};
use synth_core::graph::string::{FakerArgs, Locale};
use synth_core::{Content, Name, Namespace};
use arrow::record_batch::RecordBatch;
use arrow::array::{StringArray, ArrayRef};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct Collection {
    pub(crate) collection: Content,
}

/// Wrapper around `FieldContent` since we cant' impl `TryFrom` on a struct in a non-owned crate
struct FieldContentWrapper(Content);

pub(crate) fn build_namespace_import<T: DataSource + RelationalDataSource>(
    datasource: &T,
) -> Result<Namespace> {
    let table_names = task::block_on(datasource.get_table_names())
        .with_context(|| "Failed to get table names".to_string())?;

    let mut namespace = Namespace::default();

    info!("Building namespace collections...");
    populate_namespace_collections(&mut namespace, &table_names, datasource)?;

    info!("Building namespace primary keys...");
    populate_namespace_primary_keys(&mut namespace, &table_names, datasource)?;

    info!("Building namespace foreign keys...");
    populate_namespace_foreign_keys(&mut namespace, datasource)?;

    info!("Building namespace values...");
    populate_namespace_values(&mut namespace, &table_names, datasource)?;

    Ok(namespace)
}

fn populate_namespace_collections<T: DataSource + RelationalDataSource>(
    namespace: &mut Namespace,
    table_names: &[String],
    datasource: &T,
) -> Result<()> {
    for table_name in table_names.iter() {
        info!("Building {} collection...", table_name);

        let column_infos = task::block_on(datasource.get_columns_infos(table_name))?;

        namespace.put_collection(
            &Name::from_str(table_name)?,
            Collection::try_from((datasource, column_infos.clone()))?.collection,
        )?;

        let module = semantic_detection::module::dummy::module();
        let values = task::block_on(datasource.get_deterministic_samples(table_name))?;
        let mut pivoted = HashMap::new();
        for value in values.iter() {
            let row = value.as_object().unwrap();
            for (column, field) in row.iter() {
                if let Some(content) = field.as_str() {
                    pivoted
                        .entry(column.to_string())
                        .or_insert_with(Vec::new)
                        .push(Some(content));
                }

                if field.is_null() {
                    if let Some(values) = pivoted.get_mut(column) {
                        values.push(None);
                    }
                }
            }
        }

        let column_infos = column_infos
            .into_iter()
            .map(|ci| (ci.column_name.to_string(), ci))
            .collect::<HashMap<_, _>>();
        let pivoted = pivoted.into_iter().map(|(k, v)| (k, Arc::new(StringArray::from(v)) as ArrayRef));
        let record_batch = RecordBatch::try_from_iter(pivoted).unwrap();
        let target = module.forward(&record_batch).unwrap();
        for (column, generator) in target {
            let column_meta = column_infos.get(&column).unwrap();
            if let Some(generator) = generator {
                let field_ref = FieldRef::new(& if column_meta.is_nullable {
                    format!("{}.content.{}.0", table_name, &column_meta.column_name)
                } else {
                    format!("{}.content.{}", table_name, &column_meta.column_name)
                })?;
                if let Content::String(string_content) = namespace.get_s_node_mut(&field_ref)? {
                    *string_content = StringContent::Faker(FakerContent {
                        generator: generator.to_string(),
                        locales: vec![],
                        args: FakerArgs {
                            locales: vec![Locale::EN]
                        }
                    })
                }
            }
        }
    }

    Ok(())
}

fn populate_namespace_primary_keys<T: DataSource + RelationalDataSource>(
    namespace: &mut Namespace,
    table_names: &[String],
    datasource: &T,
) -> Result<()> {
    for table_name in table_names.iter() {
        let primary_keys = task::block_on(datasource.get_primary_keys(table_name))?;

        if primary_keys.len() > 1 {
            bail!(
                "{} primary keys found at collection {}. Synth does not currently support \
            composite primary keys.",
                primary_keys.len(),
                table_name
            )
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

    Ok(())
}

fn populate_namespace_foreign_keys<T: DataSource + RelationalDataSource>(
    namespace: &mut Namespace,
    datasource: &T,
) -> Result<()> {
    let foreign_keys = task::block_on(datasource.get_foreign_keys())?;

    debug!("{} foreign keys found.", foreign_keys.len());

    for fk in foreign_keys {
        let from_field = FieldRef::new(&format!("{}.content.{}", fk.from_table, fk.from_column))?;
        let to_field = FieldRef::new(&format!("{}.content.{}", fk.to_table, fk.to_column))?;
        let node = namespace.get_s_node_mut(&from_field)?;
        *node = Content::SameAs(SameAsContent { ref_: to_field });
    }

    Ok(())
}

fn populate_namespace_values<T: DataSource + RelationalDataSource>(
    namespace: &mut Namespace,
    table_names: &[String],
    datasource: &T,
) -> Result<()> {
    task::block_on(datasource.set_seed())?;

    for table in table_names {
        let values = task::block_on(datasource.get_deterministic_samples(table))?;

        namespace.try_update(
            OptionalMergeStrategy,
            &Name::from_str(table).unwrap(),
            &Value::from(values),
        )?;
    }

    Ok(())
}

impl<T: RelationalDataSource + DataSource> TryFrom<(&T, Vec<ColumnInfo>)> for Collection {
    type Error = anyhow::Error;

    fn try_from(columns_meta: (&T, Vec<ColumnInfo>)) -> Result<Self> {
        let mut collection = ObjectContent::default();

        for column_info in columns_meta.1 {
            let content = FieldContentWrapper::try_from((columns_meta.0, &column_info))?.0;

            collection
                .fields
                .insert(column_info.column_name.clone(), content);
        }

        Ok(Collection {
            collection: Content::Array(ArrayContent {
                length: Box::new(Content::Number(NumberContent::U64(U64::Range(RangeStep {
                    low: 1,
                    high: 2,
                    step: 1,
                })))),
                content: Box::new(Content::Object(collection)),
            }),
        })
    }
}

impl<T: RelationalDataSource + DataSource> TryFrom<(&T, &ColumnInfo)> for FieldContentWrapper {
    type Error = anyhow::Error;

    fn try_from(column_meta: (&T, &ColumnInfo)) -> Result<Self> {
        let data_type = &column_meta.1.data_type;
        let mut content = column_meta
            .0
            .decode_to_content(data_type, column_meta.1.character_maximum_length)?;

        if column_meta.1.is_nullable {
            content = content.into_nullable();
        }

        Ok(FieldContentWrapper(content))
    }
}
