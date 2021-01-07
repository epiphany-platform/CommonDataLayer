use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::PathBuf;

use anyhow::Context;
use log::trace;
use rpc::document_storage::document_storage_server::{DocumentStorage, DocumentStorageServer};
use rpc::document_storage::{
    DataMap, Empty, RetrieveBySchemaRequest, RetrieveMultipleRequest, StoreRequest,
};
use sled::{Db, IVec};
use std::collections::HashMap;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::GRPC_PORT;
use utils::metrics::counter;

struct Connector {
    db: Db,
}

struct StorageValue {
    schema_id: Uuid,
    data: Vec<u8>,
}

impl StorageValue {
    pub fn into_vec(self) -> Vec<u8> {
        let mut data = self.data;
        data.extend_from_slice(self.schema_id.as_bytes());

        data
    }

    pub fn from_ivec(value: IVec) -> Self {
        let schema_id = Self::uuid_from_slice(&value[(value.len() - 16)..]);
        let mut data = value.to_vec();
        data.truncate(data.len() - 16);

        Self { schema_id, data }
    }

    fn uuid_from_slice(slice: &[u8]) -> Uuid {
        let mut bytes = [0u8; 16];
        bytes.clone_from_slice(slice);

        Uuid::from_bytes(bytes)
    }
}

impl Connector {
    fn store(&self, object_id: Uuid, value: StorageValue) -> Result<(), Status> {
        self.db
            .insert(object_id.as_bytes(), value.into_vec())
            .map_err(error_to_status)?;

        Ok(())
    }

    fn retrieve(&self, object_id: Uuid) -> Result<Vec<u8>, Status> {
        match self.db.get(object_id.as_bytes()) {
            Ok(Some(value)) => Ok(StorageValue::from_ivec(value).data),
            Ok(None) => Err(Status::not_found(format!(
                "Value not found for object ID {}",
                object_id
            ))),
            Err(error) => Err(error_to_status(error)),
        }
    }

    fn retrieve_by_schema(&self, schema_id: Uuid) -> Result<HashMap<String, Vec<u8>>, Status> {
        self.db
            .iter()
            .filter_map(|res| {
                res.map_err(error_to_status)
                    .map(|(key, value)| Self::filter_row_by_schema_id(key, value, schema_id))
                    .transpose()
            })
            .collect()
    }

    fn filter_row_by_schema_id(
        key: IVec,
        value: IVec,
        schema_id: Uuid,
    ) -> Option<(String, Vec<u8>)> {
        let value = StorageValue::from_ivec(value);

        if schema_id == value.schema_id {
            Some((
                Uuid::from_slice(&key[..]).unwrap_or_default().to_string(),
                value.data,
            ))
        } else {
            None
        }
    }
}

#[tonic::async_trait]
impl DocumentStorage for Connector {
    async fn store(&self, request: Request<StoreRequest>) -> Result<Response<Empty>, Status> {
        counter!("cdl.document-storage.store", 1);
        let request = request.into_inner();

        let object_id = parse_object_id(&request.object_id)?;
        let schema_id = parse_schema_id(&request.schema_id)?;
        trace!("Received store gRPC request {:?}", object_id);

        self.store(
            object_id,
            StorageValue {
                schema_id,
                data: request.data,
            },
        )?;

        trace!("Finished store gRPC request {:?}", object_id);

        Ok(Response::new(Empty {}))
    }

    async fn retrieve_multiple(
        &self,
        request: Request<RetrieveMultipleRequest>,
    ) -> Result<Response<DataMap>, Status> {
        counter!("cdl.document-storage.retrieve-multiple", 1);
        let object_ids = request
            .into_inner()
            .object_ids
            .iter()
            .map(|id| parse_object_id(id))
            .collect::<Result<Vec<Uuid>, Status>>()?;

        let values = object_ids
            .into_iter()
            .map(|object_id| {
                self.retrieve(object_id)
                    .map(|value| (object_id.to_string(), value))
            })
            .collect::<Result<HashMap<String, Vec<u8>>, Status>>()?;

        Ok(Response::new(DataMap { values }))
    }

    async fn retrieve_by_schema(
        &self,
        request: Request<RetrieveBySchemaRequest>,
    ) -> Result<Response<DataMap>, Status> {
        counter!("cdl.document-storage.retrieve-by-schema", 1);
        let schema_id = parse_schema_id(&request.into_inner().schema_id)?;

        let values = self.retrieve_by_schema(schema_id)?;

        Ok(Response::new(DataMap { values }))
    }
}

pub async fn run(datastore_root: PathBuf) -> anyhow::Result<()> {
    let db = sled::open(datastore_root)?;

    Server::builder()
        .add_service(DocumentStorageServer::new(Connector { db }))
        .serve(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), GRPC_PORT).into())
        .await
        .context("Couldn't spawn gRPC server")
}

fn parse_object_id(object_id: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(object_id).map_err(|err| {
        Status::invalid_argument(format!("Invalid UUID provided for object ID: {}", err))
    })
}

fn parse_schema_id(schema_id: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(schema_id).map_err(|err| {
        Status::invalid_argument(format!("Invalid UUID provided for schema ID: {}", err))
    })
}

fn error_to_status(err: sled::Error) -> tonic::Status {
    tonic::Status::internal(err.to_string())
}
