use crate::error::ClientError;
use edge_registry_client::EdgeRegistryClient;
use tonic::transport::Channel;

pub use crate::codegen::edge_registry::*;
use bb8::{Pool, PooledConnection};

pub type EdgeRegistryPool = Pool<EdgeRegistryConnectionManager>;
pub type EdgeRegistryConn = EdgeRegistryClient<Channel>;

pub struct EdgeRegistryConnectionManager {
    pub address: String,
}

pub async fn connect(addr: String) -> Result<EdgeRegistryClient<Channel>, ClientError> {
    connect_inner(addr)
        .await
        .map_err(|err| ClientError::ConnectionError { source: err })
}

async fn connect_inner(
    addr: String,
) -> Result<EdgeRegistryClient<Channel>, tonic::transport::Error> {
    let conn = tonic::transport::Endpoint::new(addr)?.connect().await?;

    Ok(EdgeRegistryClient::with_interceptor(
        conn,
        tracing_utils::grpc::interceptor(),
    ))
}

#[async_trait::async_trait]
impl bb8::ManageConnection for EdgeRegistryConnectionManager {
    type Connection = EdgeRegistryConn;
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
