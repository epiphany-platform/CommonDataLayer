use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::PathBuf;

use anyhow::Context;
use log::trace;
use sled::Db;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::GRPC_PORT;
use rpc::blob_storage::blob_storage_server::{BlobStorage, BlobStorageServer};
use rpc::blob_storage::{Empty, RetrieveRequest, RetrieveResponse, StoreRequest};
use utils::metrics::counter;

struct Connector {
    db: Db,
}

impl Connector {
    fn store(&self, object_id: Uuid, value: Vec<u8>) -> Result<(), Status> {
        self.db
            .insert(object_id.as_bytes(), value)
            .map_err(error_to_status)?;

        Ok(())
    }

    fn retrieve(&self, object_id: Uuid) -> Result<Vec<u8>, Status> {
        match self.db.get(object_id.as_bytes()) {
            Ok(Some(value)) => Ok(value.to_vec()),
            Ok(None) => Err(Status::not_found(format!(
                "Value not found for object ID {}",
                object_id
            ))),
            Err(error) => Err(error_to_status(error)),
        }
    }
}

#[tonic::async_trait]
impl BlobStorage for Connector {
    async fn store(&self, request: Request<StoreRequest>) -> Result<Response<Empty>, Status> {
        counter!("cdl.blob-store.store", 1);
        let request = request.into_inner();

        let object_id = parse_object_id(&request.object_id)?;
        trace!("Received store gRPC request {:?}", object_id);

        self.store(object_id, request.data)?;

        trace!("Finished store gRPC request {:?}", object_id);

        Ok(Response::new(Empty {}))
    }

    async fn retrieve(
        &self,
        request: Request<RetrieveRequest>,
    ) -> Result<Response<RetrieveResponse>, Status> {
        counter!("cdl.blob-store.retrieve", 1);
        let object_id = parse_object_id(&request.into_inner().object_id)?;

        let data = self.retrieve(object_id)?;

        Ok(Response::new(RetrieveResponse { data }))
    }
}

pub async fn run(datastore_root: PathBuf) -> anyhow::Result<()> {
    let db = sled::open(datastore_root)?;

    Server::builder()
        .add_service(BlobStorageServer::new(Connector { db }))
        .serve(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), GRPC_PORT).into())
        .await
        .context("Couldn't spawn gRPC server")
}

fn parse_object_id(object_id: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(object_id).map_err(|err| {
        Status::invalid_argument(format!("Invalid UUID provided for object ID: {}", err))
    })
}

fn error_to_status(err: sled::Error) -> tonic::Status {
    tonic::Status::internal(err.to_string())
}
