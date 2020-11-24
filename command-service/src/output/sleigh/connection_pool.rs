use document_storage::grpc::schema::storage_client::StorageClient;
use tonic::transport::Channel;

pub struct SleighConnectionManager {
    pub addr: String,
}

#[async_trait::async_trait]
impl bb8::ManageConnection for SleighConnectionManager {
    type Connection = StorageClient<Channel>;
    type Error = tonic::transport::Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        Ok(StorageClient::connect(self.addr.clone()).await?)
    }

    async fn is_valid(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}
