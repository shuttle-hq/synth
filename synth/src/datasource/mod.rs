use serde_json::Value;
use anyhow::Result;
use async_trait::async_trait;

pub(crate) mod relational_datasource;
pub(crate) mod postgres_datasource;
pub(crate) mod mysql_datasource;

#[async_trait]
pub trait DataSource {
    type ConnectParams;

    fn new(connect_params: &Self::ConnectParams) -> Result<Self> where Self: Sized;

    async fn insert_data(
        &self,
        collection_name: String,
        collection: &[Value],
    ) -> Result<()>;
}