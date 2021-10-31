use anyhow::Result;
use async_trait::async_trait;
use synth_core::Value;

pub(crate) mod mysql_datasource;
pub(crate) mod postgres_datasource;
pub(crate) mod relational_datasource;

/// This trait encompasses all data source types, whether it's SQL or No-SQL. APIs should be defined
/// async when possible, delegating to the caller on how to handle it. Data source specific
/// implementations should be defined within the implementing struct.
#[async_trait]
pub trait DataSource {
    type ConnectParams;

    fn new(connect_params: &Self::ConnectParams) -> Result<Self>
    where
        Self: Sized;

    async fn insert_data(&self, collection_name: &str, collection: &[Value]) -> Result<()>;
}
