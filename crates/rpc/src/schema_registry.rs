pub mod types;

pub use crate::codegen::schema_registry::*;

use crate::error::ClientError;
use bb8::{Pool, PooledConnection};
use schema_registry_client::SchemaRegistryClient;
use tonic::transport::Channel;

pub type SchemaRegistryPool = Pool<SchemaRegistryConnectionManager>;
pub type SchemaRegistryConn = SchemaRegistryClient<Channel>;

pub struct SchemaRegistryConnectionManager {
    pub address: String,
}

pub async fn connect(addr: String) -> Result<SchemaRegistryClient<Channel>, ClientError> {
    connect_inner(addr)
        .await
        .map_err(|err| ClientError::ConnectionError { source: err })
}

async fn connect_inner(
    addr: String,
) -> Result<SchemaRegistryClient<Channel>, tonic::transport::Error> {
    let conn = tonic::transport::Endpoint::new(addr)?.connect().await?;

    Ok(SchemaRegistryClient::with_interceptor(
        conn,
        tracing_utils::grpc::interceptor(),
    ))
}

#[async_trait::async_trait]
impl bb8::ManageConnection for SchemaRegistryConnectionManager {
    type Connection = SchemaRegistryConn;
    type Error = ClientError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        tracing::debug!("Connecting to registry");

        connect(self.address.clone()).await
    }

    async fn is_valid(&self, conn: &mut PooledConnection<'_, Self>) -> Result<(), Self::Error> {
        conn.heartbeat(Empty {})
            .await
            .map_err(|source| ClientError::QueryError { source })?;

        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}
