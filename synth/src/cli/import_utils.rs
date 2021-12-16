use crate::datasource::relational_datasource::{
    ColumnInfo, ForeignKey, PrimaryKey, RelationalDataSource, SqlxDataSource, ValueWrapper,
};
use crate::datasource::DataSource;
use anyhow::{Context, Result};
use async_std::task;
use log::debug;
use serde_json::Value;
use sqlx::{Database, Executor, Row};
use std::convert::TryFrom;
use synth_core::graph::json::synth_val_to_json;
use synth_core::schema::content::number_content::U64;
use synth_core::schema::{
    ArrayContent, FieldRef, NumberContent, ObjectContent, OptionalMergeStrategy, RangeStep,
    SameAsContent, UniqueContent,
};
use synth_core::{Content, Namespace};

#[derive(Debug)]
pub(crate) struct Collection {
    pub(crate) collection: Content,
}

/// Wrapper around `FieldContent` since we cant' impl `TryFrom` on a struct in a non-owned crate
struct FieldContentWrapper(Content);

pub(crate) fn build_namespace_import<T: DataSource + RelationalDataSource + SqlxDataSource>(
    datasource: &T,
) -> Result<Namespace>
where
    T: Sync,
    for<'c> &'c mut <T::DB as Database>::Connection: Executor<'c, Database = T::DB>,
    String: sqlx::Type<T::DB>,
    for<'d> String: sqlx::Decode<'d, T::DB> + sqlx::Encode<'d, T::DB>,
    usize: sqlx::ColumnIndex<<T::DB as sqlx::Database>::Row>,
    PrimaryKey: TryFrom<<T::DB as sqlx::Database>::Row, Error = anyhow::Error>,
    ForeignKey: TryFrom<<T::DB as sqlx::Database>::Row, Error = anyhow::Error>,
    ValueWrapper: TryFrom<<T::DB as sqlx::Database>::Row, Error = anyhow::Error>,
{
    let table_names = task::block_on(get_table_names(datasource))
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

async fn get_table_names<T: SqlxDataSource>(datasource: &T) -> Result<Vec<String>>
where
    for<'c> &'c mut <T::DB as Database>::Connection: Executor<'c, Database = T::DB>,
    String: sqlx::Type<T::DB>,
    for<'d> String: sqlx::Decode<'d, T::DB>,
    usize: sqlx::ColumnIndex<<T::DB as sqlx::Database>::Row>,
{
    let query = datasource.get_table_names_query();
    let pool = datasource.get_pool();

    let rows = datasource.query(query).fetch_all(&pool).await?;

    let table_names = rows
        .into_iter()
        .map(|row| row.get::<String, usize>(0))
        .collect();

    Ok(table_names)
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
            table_name.clone(),
            Collection::try_from((datasource, column_infos))?.collection,
        )?;
    }

    Ok(())
}

fn populate_namespace_primary_keys<T: DataSource + RelationalDataSource + SqlxDataSource>(
    namespace: &mut Namespace,
    table_names: &[String],
    datasource: &T,
) -> Result<()>
where
    for<'c> &'c mut <T::DB as Database>::Connection: Executor<'c, Database = T::DB>,
    String: sqlx::Type<T::DB>,
    for<'d> String: sqlx::Encode<'d, T::DB>,
    PrimaryKey: TryFrom<<T::DB as sqlx::Database>::Row, Error = anyhow::Error>,
{
    for table_name in table_names.iter() {
        let primary_keys = task::block_on(get_primary_keys(datasource, table_name.to_string()))?;

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

async fn get_primary_keys<T: SqlxDataSource>(
    datasource: &T,
    table_name: String,
) -> Result<Vec<PrimaryKey>>
where
    for<'c> &'c mut <T::DB as Database>::Connection: Executor<'c, Database = T::DB>,
    String: sqlx::Type<T::DB>,
    for<'d> String: sqlx::Encode<'d, T::DB>,
    PrimaryKey: TryFrom<<T::DB as sqlx::Database>::Row, Error = anyhow::Error>,
{
    let query = datasource.get_primary_keys_query();
    let pool = datasource.get_pool();

    datasource
        .query(query)
        .bind(table_name)
        .fetch_all(&pool)
        .await?
        .into_iter()
        .map(PrimaryKey::try_from)
        .collect()
}

fn populate_namespace_foreign_keys<T: DataSource + RelationalDataSource + SqlxDataSource>(
    namespace: &mut Namespace,
    datasource: &T,
) -> Result<()>
where
    for<'c> &'c mut <T::DB as Database>::Connection: Executor<'c, Database = T::DB>,
    ForeignKey: TryFrom<<T::DB as sqlx::Database>::Row, Error = anyhow::Error>,
{
    let foreign_keys = task::block_on(get_foreign_keys(datasource))?;

    debug!("{} foreign keys found.", foreign_keys.len());

    for fk in foreign_keys {
        let from_field = FieldRef::new(&format!("{}.content.{}", fk.from_table, fk.from_column))?;
        let to_field = FieldRef::new(&format!("{}.content.{}", fk.to_table, fk.to_column))?;
        let node = namespace.get_s_node_mut(&from_field)?;
        *node = Content::SameAs(SameAsContent { ref_: to_field });
    }

    Ok(())
}

async fn get_foreign_keys<T: SqlxDataSource>(datasource: &T) -> Result<Vec<ForeignKey>>
where
    for<'c> &'c mut <T::DB as Database>::Connection: Executor<'c, Database = T::DB>,
    ForeignKey: TryFrom<<T::DB as sqlx::Database>::Row, Error = anyhow::Error>,
{
    let query = datasource.get_foreign_keys_query();
    let pool = datasource.get_pool();

    datasource
        .query(query)
        .fetch_all(&pool)
        .await?
        .into_iter()
        .map(ForeignKey::try_from)
        .collect()
}

fn populate_namespace_values<T: DataSource + RelationalDataSource + SqlxDataSource>(
    namespace: &mut Namespace,
    table_names: &[String],
    datasource: &T,
) -> Result<()>
where
    T: Sync,
    for<'c> &'c mut <T::DB as Database>::Connection: Executor<'c, Database = T::DB>,
    String: sqlx::Type<T::DB>,
    for<'d> String: sqlx::Encode<'d, T::DB>,
    ValueWrapper: TryFrom<<T::DB as sqlx::Database>::Row, Error = anyhow::Error>,
{
    task::block_on(datasource.set_seed())?;

    for table_name in table_names {
        let values = task::block_on(get_deterministic_samples(
            datasource,
            table_name.to_string(),
        ))?;
        let json_values: Vec<Value> = values.into_iter().map(synth_val_to_json).collect();
        namespace.try_update(OptionalMergeStrategy, table_name, &Value::from(json_values))?;
    }

    Ok(())
}

async fn get_deterministic_samples<T: SqlxDataSource>(
    datasource: &T,
    table: String,
) -> Result<Vec<synth_core::Value>>
where
    for<'c> &'c mut <T::DB as Database>::Connection: Executor<'c, Database = T::DB>,
    ValueWrapper: TryFrom<<T::DB as sqlx::Database>::Row, Error = anyhow::Error>,
{
    let query = datasource.get_deterministic_samples_query(table);
    let pool = datasource.get_pool();

    datasource
        .query(&query)
        .fetch_all(&pool)
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
        .collect()
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
