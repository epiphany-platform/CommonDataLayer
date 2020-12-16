use bb8::PooledConnection;
use rpc::document_storage::document_storage_client::DocumentStorageClient;
use tonic::transport::Channel;

pub struct SleighConnectionManager {
    pub addr: String,
}

#[async_trait::async_trait]
impl bb8::ManageConnection for SleighConnectionManager {
    type Connection = DocumentStorageClient<Channel>;
    type Error = tonic::transport::Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        Ok(DocumentStorageClient::connect(self.addr.clone()).await?)
    }

    async fn is_valid(&self, _: &mut PooledConnection<'_, Self>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}
