use anyhow::Context;
use bb8::{Pool, PooledConnection};
use rpc::document_storage::document_storage_client::DocumentStorageClient;
use rpc::document_storage::{RetrieveBySchemaRequest, RetrieveMultipleRequest};
use rpc::query_service::{
    query_service_server::QueryService, ObjectIds, RawStatement, SchemaId, ValueBytes, ValueMap,
};
use structopt::StructOpt;
use tonic::transport::Channel;
use tonic::{Code, Request, Response, Status};
use utils::metrics::counter;

#[derive(Debug, StructOpt)]
pub struct DsConfig {
    #[structopt(long = "query-url", env = "DS_QUERY_URL")]
    ds_url: String,
}

pub struct DsConnectionManager {
    pub addr: String,
}

#[tonic::async_trait]
impl bb8::ManageConnection for DsConnectionManager {
    type Connection = DocumentStorageClient<Channel>;
    type Error = tonic::transport::Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        Ok(DocumentStorageClient::connect(self.addr.clone()).await?)
    }

    async fn is_valid(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

pub struct DsQuery {
    pool: Pool<DsConnectionManager>,
}

impl DsQuery {
    pub async fn load(config: DsConfig) -> anyhow::Result<Self> {
        let manager = DsConnectionManager {
            addr: config.ds_url,
        };
        let pool = Pool::builder()
            .build(manager)
            .await
            .context("Failed to build connection pool")?;

        Ok(Self { pool })
    }

    async fn connect(&self) -> Result<PooledConnection<'_, DsConnectionManager>, Status> {
        self.pool
            .get()
            .await
            .map_err(|err| Status::internal(format!("Unable to connect to database: {}", err)))
    }
}

#[tonic::async_trait]
impl QueryService for DsQuery {
    async fn query_multiple(
        &self,
        request: Request<ObjectIds>,
    ) -> Result<Response<ValueMap>, Status> {
        counter!("cdl.query-service.query-multiple.sled", 1);
        let mut conn = self.connect().await?;
        let response = conn
            .retrieve_multiple(RetrieveMultipleRequest {
                object_ids: request.into_inner().object_ids,
            })
            .await?;

        Ok(tonic::Response::new(ValueMap {
            values: response.into_inner().values,
        }))
    }

    async fn query_by_schema(
        &self,
        request: Request<SchemaId>,
    ) -> Result<Response<ValueMap>, Status> {
        counter!("cdl.query-service.query-by-schema.sled", 1);
        let mut conn = self.connect().await?;
        let response = conn
            .retrieve_by_schema(RetrieveBySchemaRequest {
                schema_id: request.into_inner().schema_id,
            })
            .await?;

        Ok(tonic::Response::new(ValueMap {
            values: response.into_inner().values,
        }))
    }

    async fn query_raw(
        &self,
        _request: Request<RawStatement>,
    ) -> Result<Response<ValueBytes>, Status> {
        counter!("cdl.query-service.query_raw.sled", 1);

        Err(Status::new(
            Code::Unimplemented,
            "query-service-sled does not support RAW requests yet",
        ))
    }
}
