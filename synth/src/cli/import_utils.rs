use crate::datasource::relational_datasource::{ColumnInfo, RelationalDataSource, PrimaryKey, ForeignKey};
use synth_core::{Namespace, Name, Content};
use crate::datasource::DataSource;
use async_std::task;
use std::str::FromStr;
use anyhow::{Result, Context};
use log::debug;
use synth_core::schema::{FieldRef, NumberContent, Id, SameAsContent, OptionalMergeStrategy};
use synth_core::schema::content::number_content::U64;
use serde_json::Value;

#[derive(Debug)]
pub(crate) struct Collection {
    pub(crate) collection: Content,
}

pub(crate) fn build_namespace_import<T: DataSource + RelationalDataSource>(datasource: &T)
    -> Result<Namespace> {
    let table_names = task::block_on(async {
        let table_names = datasource.get_table_names().await?;
        Ok::<Vec<String>, anyhow::Error>(table_names)
    }).context(format!("Failed to get table names"))?;

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
    namespace: &mut Namespace, table_names: &Vec<String>, datasource: &T) -> Result<()> {
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

    Ok(())
}

fn populate_namespace_primary_keys<T: DataSource + RelationalDataSource>(
    namespace: &mut Namespace, table_names: &Vec<String>, datasource: &T) -> Result<()> {
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

    Ok(())
}

fn populate_namespace_foreign_keys<T: DataSource + RelationalDataSource>(
    namespace: &mut Namespace, datasource: &T) -> Result<()> {
    let foreign_keys = task::block_on(async {
        Ok::<Vec<ForeignKey>, anyhow::Error>(datasource.get_foreign_keys().await?)
    })?;

    debug!("{} foreign keys found.", foreign_keys.len());

    for fk in foreign_keys {
        let from_field =
            FieldRef::new(&format!("{}.content.{}", fk.from_table, fk.from_column))?;
        let to_field = FieldRef::new(&format!("{}.content.{}", fk.to_table, fk.to_column))?;
        let node = namespace.get_s_node_mut(&from_field)?;
        *node = Content::SameAs(SameAsContent { ref_: to_field });
    }

    Ok(())
}

fn populate_namespace_values<T: DataSource + RelationalDataSource>(
    namespace: &mut Namespace, table_names: &Vec<String>, datasource: &T) -> Result<()> {
    for table in table_names {
        let values = task::block_on(async {
            Ok::<Vec<Value>, anyhow::Error>(datasource.get_deterministic_samples(&table).await?)
        })?;

        namespace.try_update(
            OptionalMergeStrategy,
            &Name::from_str(&table).unwrap(),
            &Value::from(values),
        )?;
    }

    Ok(())
}