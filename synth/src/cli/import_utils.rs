use crate::datasource::relational_datasource::{ColumnInfo, RelationalDataSource};
use crate::datasource::DataSource;
use anyhow::{Context, Result};
use async_std::task;
use log::debug;
use serde_json::Value;
use std::convert::TryFrom;
use synth_core::graph::json::synth_val_to_json;
use synth_core::schema::content::number_content::U64;
use synth_core::schema::{
    ArrayContent, FieldRef, MergeStrategy, NumberContent, ObjectContent, OptionalMergeStrategy,
    RangeStep, SameAsContent, UniqueContent,
};
use synth_core::Content;

#[derive(Debug)]
pub(crate) struct Collection {
    pub(crate) collection: Content,
}

/// Wrapper around `FieldContent` since we cant' impl `TryFrom` on a struct in a non-owned crate
struct FieldContentWrapper(Content);

pub(crate) fn build_namespace_import<T: DataSource + RelationalDataSource>(
    datasource: &T,
) -> Result<Content> {
    let table_names = task::block_on(datasource.get_table_names())
        .with_context(|| "Failed to get table names".to_string())?;

    let mut namespace = Content::new_object();

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
    namespace: &mut Content,
    table_names: &[String],
    datasource: &T,
) -> Result<()> {
    for table_name in table_names.iter() {
        info!("Building {} collection...", table_name);

        let column_infos = task::block_on(datasource.get_columns_infos(table_name))?;

        namespace.put_collection(
            table_name.clone(),
            Collection::try_from((datasource, column_infos))?.collection,
        )?;
    }

    Ok(())
}

fn populate_namespace_primary_keys<T: DataSource + RelationalDataSource>(
    namespace: &mut Content,
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
            // if the primary key is a number, use an id generator.
            let pk_node = match node {
                Content::Number(n) => n.clone().try_transmute_to_id().ok().map(Content::Number),
                _ => None,
            };

            *node = pk_node.unwrap_or_else(|| {
                Content::Unique(UniqueContent {
                    algorithm: Default::default(),
                    content: Box::new(node.clone()),
                })
            });
        }
    }

    Ok(())
}

fn populate_namespace_foreign_keys<T: DataSource + RelationalDataSource>(
    namespace: &mut Content,
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
    namespace: &mut Content,
    table_names: &[String],
    datasource: &T,
) -> Result<()> {
    task::block_on(datasource.set_seed())?;

    for table_name in table_names {
        let values = task::block_on(datasource.get_deterministic_samples(table_name))?;
        let json_values: Vec<Value> = values.into_iter().map(synth_val_to_json).collect();
        OptionalMergeStrategy.try_merge(
            namespace.get_collection_mut(table_name)?,
            &Value::from(json_values),
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
                length: Box::new(Content::Number(NumberContent::U64(U64::Range(
                    RangeStep::new(1, 2, 1),
                )))),
                content: Box::new(Content::Object(collection)),
            }),
        })
    }
}

impl<T: RelationalDataSource + DataSource> TryFrom<(&T, &ColumnInfo)> for FieldContentWrapper {
    type Error = anyhow::Error;

    fn try_from(column_meta: (&T, &ColumnInfo)) -> Result<Self> {
        let mut content = column_meta.0.decode_to_content(column_meta.1)?;

        if column_meta.1.is_nullable {
            content = content.into_nullable();
        }

        Ok(FieldContentWrapper(content))
    }
}
