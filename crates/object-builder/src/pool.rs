use bb8::{Pool, PooledConnection};
use rpc::edge_registry::edge_registry_client::EdgeRegistryClient;
use rpc::schema_registry::schema_registry_client::SchemaRegistryClient;
use tonic::transport::Channel;

pub type SchemaRegistryPool = Pool<SchemaRegistryConnectionManager>;
pub type SchemaRegistryConn = SchemaRegistryClient<Channel>;

pub type EdgeRegistryPool = Pool<EdgeRegistryConnectionManager>;
pub type EdgeRegistryConn = EdgeRegistryClient<Channel>;

#[derive(Clone)]
pub struct SchemaRegistryConnectionManager {
    pub address: String,
}

#[async_trait::async_trait]
impl bb8::ManageConnection for SchemaRegistryConnectionManager {
    type Connection = SchemaRegistryConn;
    type Error = rpc::error::ClientError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        tracing::debug!("Connecting to registry");

        rpc::schema_registry::connect(self.address.clone()).await
    }

    async fn is_valid(&self, conn: &mut PooledConnection<'_, Self>) -> Result<(), Self::Error> {
        conn.ping(rpc::schema_registry::Empty {})
            .await
            .map_err(|source| rpc::error::ClientError::QueryError { source })?;

        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}
#[derive(Clone)]
pub struct EdgeRegistryConnectionManager {
    pub address: String,
}

#[async_trait::async_trait]
impl bb8::ManageConnection for EdgeRegistryConnectionManager {
    type Connection = EdgeRegistryConn;
    type Error = rpc::error::ClientError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        tracing::debug!("Connecting to registry");

        rpc::edge_registry::connect(self.address.clone()).await
    }

    async fn is_valid(&self, conn: &mut PooledConnection<'_, Self>) -> Result<(), Self::Error> {
        conn.heartbeat(rpc::edge_registry::Empty {})
            .await
            .map_err(|source| rpc::error::ClientError::QueryError { source })?;

        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}
