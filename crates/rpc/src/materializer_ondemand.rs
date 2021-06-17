use crate::error::ClientError;
use on_demand_materializer_client::OnDemandMaterializerClient;
use tonic::transport::Channel;

pub use crate::codegen::materializer_ondemand::*;
use bb8::{Pool, PooledConnection};

pub type OnDemandMaterializerConn = OnDemandMaterializerClient<Channel>;
pub type OnDemandMaterializerPool = Pool<OnDemandMaterializerConnectionManager>;

pub struct OnDemandMaterializerConnectionManager {
    pub address: String,
}

pub async fn connect(addr: String) -> Result<OnDemandMaterializerClient<Channel>, ClientError> {
    connect_inner(addr)
        .await
        .map_err(|err| ClientError::ConnectionError { source: err })
}

async fn connect_inner(
    addr: String,
) -> Result<OnDemandMaterializerClient<Channel>, tonic::transport::Error> {
    let conn = tonic::transport::Endpoint::new(addr)?.connect().await?;

    Ok(OnDemandMaterializerClient::with_interceptor(
        conn,
        tracing_utils::grpc::interceptor(),
    ))
}

#[async_trait::async_trait]
impl bb8::ManageConnection for OnDemandMaterializerConnectionManager {
    type Connection = OnDemandMaterializerConn;
    type Error = ClientError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        tracing::debug!("Connecting to object builder");

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
