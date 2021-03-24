use std::convert::TryInto;
use std::pin::Pin;

use anyhow::Context;
use semver::Version;
use semver::VersionReq;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::db::SchemaRegistryDb;
use crate::error::RegistryError;
use crate::types::{DbExport, NewSchema, SchemaDefinition, SchemaUpdate, VersionedUuid};
use crate::CommunicationMethodConfig;
use rpc::schema_registry::{
    schema_registry_server::SchemaRegistry, Empty, Errors, Id, SchemaMetadataUpdate,
    ValueToValidate, VersionedId,
};
use utils::communication::metadata_fetcher::MetadataFetcher;
use utils::communication::Result;

pub struct SchemaRegistryImpl {
    pub db: SchemaRegistryDb,
    pub mq_metadata: MetadataFetcher,
}

impl SchemaRegistryImpl {
    pub async fn new(
        db_url: String,
        communication_config: CommunicationMethodConfig,
    ) -> anyhow::Result<Self> {
        let db = SchemaRegistryDb::connect(db_url).await?;
        let mq_metadata = match &communication_config {
            CommunicationMethodConfig::Kafka(kafka) => {
                MetadataFetcher::new_kafka(&kafka.brokers).await?
            }
            CommunicationMethodConfig::Amqp(amqp) => {
                MetadataFetcher::new_amqp(&amqp.connection_string).await?
            }
            CommunicationMethodConfig::Grpc => MetadataFetcher::new_grpc("command_service").await?,
        };

        Ok(Self { db, mq_metadata })
    }

    pub async fn export_all(&self) -> anyhow::Result<DbExport> {
        self.db
            .export_all()
            .await
            .context("Failed to export the entire database")
    }

    pub async fn import_all(&self, imported: DbExport) -> anyhow::Result<()> {
        self.db
            .import_all(imported)
            .await
            .context("failed to import database")
    }
}

#[tonic::async_trait]
impl SchemaRegistry for SchemaRegistryImpl {
    #[tracing::instrument(skip(self))]
    async fn add_schema(
        &self,
        request: Request<rpc::schema_registry::NewSchema>,
    ) -> Result<Response<Id>, Status> {
        let request = request.into_inner();
        let new_schema = NewSchema {
            name: request.metadata.name,
            definition: parse_json_and_deserialize(&request.definition)?,
            query_address: request.metadata.query_address,
            insert_destination: request.metadata.insert_destination,
            r#type: request.metadata.r#type.try_into()?,
        };

        if !new_schema.insert_destination.is_empty()
            && !self
                .mq_metadata
                .destination_exists(&new_schema.insert_destination)
                .await
                .map_err(RegistryError::from)?
        {
            return Err(RegistryError::NoTopic(new_schema.insert_destination.clone()).into());
        }

        let new_id = self.db.add_schema(new_schema).await?;

        Ok(Response::new(Id {
            id: new_id.to_string(),
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn add_schema_version(
        &self,
        request: Request<rpc::schema_registry::NewSchemaVersion>,
    ) -> Result<Response<Empty>, Status> {
        let request = request.into_inner();
        let schema_id = parse_uuid(&request.id)?;
        let new_version = SchemaDefinition {
            version: parse_version(&request.definition.version)?,
            definition: parse_json_and_deserialize(&request.definition.definition)?,
        };

        self.db
            .add_new_version_of_schema(schema_id, new_version)
            .await?;

        Ok(Response::new(Empty {}))
    }

    #[tracing::instrument(skip(self))]
    async fn update_schema(
        &self,
        request: Request<SchemaMetadataUpdate>,
    ) -> Result<Response<Empty>, Status> {
        let request = request.into_inner();
        let schema_id = parse_uuid(&request.id)?;

        let schema_type = if let Some(st) = request.patch.r#type {
            Some(st.try_into()?)
        } else {
            None
        };

        if let Some(destination) = request.patch.insert_destination.as_ref() {
            if !self
                .mq_metadata
                .destination_exists(&destination)
                .await
                .map_err(RegistryError::from)?
            {
                return Err(RegistryError::NoTopic(destination.clone()).into());
            }
        }

        self.db
            .update_schema(
                schema_id,
                SchemaUpdate {
                    name: request.patch.name,
                    query_address: request.patch.query_address,
                    insert_destination: request.patch.insert_destination,
                    r#type: schema_type,
                },
            )
            .await?;

        Ok(Response::new(Empty {}))
    }

    #[tracing::instrument(skip(self))]
    async fn get_schema_metadata(
        &self,
        request: Request<Id>,
    ) -> Result<Response<rpc::schema_registry::SchemaMetadata>, Status> {
        let request = request.into_inner();
        let id = parse_uuid(&request.id)?;

        let schema = self.db.get_schema(id).await?;

        Ok(Response::new(rpc::schema_registry::SchemaMetadata {
            name: schema.name,
            insert_destination: schema.insert_destination,
            query_address: schema.query_address,
            r#type: schema.r#type.into(),
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn get_schema_definition(
        &self,
        request: Request<VersionedId>,
    ) -> Result<Response<rpc::schema_registry::SchemaDefinition>, Status> {
        let request = request.into_inner();
        let versioned_id = VersionedUuid {
            id: parse_uuid(&request.id)?,
            version_req: parse_optional_version_req(&request.version_req)?
                .unwrap_or_else(VersionReq::any),
        };

        let (version, definition) = self.db.get_schema_definition(&versioned_id).await?;

        Ok(Response::new(rpc::schema_registry::SchemaDefinition {
            version: version.to_string(),
            definition: serialize_json(&definition)?,
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn get_schema_versions(
        &self,
        request: Request<Id>,
    ) -> Result<Response<rpc::schema_registry::SchemaVersions>, Status> {
        let request = request.into_inner();
        let id = parse_uuid(&request.id)?;

        let versions = self.db.get_schema_versions(id).await?;

        Ok(Response::new(rpc::schema_registry::SchemaVersions {
            versions: versions.into_iter().map(|v| v.to_string()).collect(),
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn get_schema_with_definitions(
        &self,
        request: Request<Id>,
    ) -> Result<Response<rpc::schema_registry::SchemaWithDefinitions>, Status> {
        let request = request.into_inner();
        let id = parse_uuid(&request.id)?;

        let schema = self.db.get_schema_with_definitions(id).await?;

        Ok(Response::new(rpc::schema_registry::SchemaWithDefinitions {
            id: request.id,
            metadata: rpc::schema_registry::SchemaMetadata {
                name: schema.name,
                insert_destination: schema.insert_destination,
                query_address: schema.query_address,
                r#type: schema.r#type.into(),
            },
            definitions: schema
                .definitions
                .into_iter()
                .map(|definition| {
                    Ok(rpc::schema_registry::SchemaDefinition {
                        version: definition.version.to_string(),
                        definition: serialize_json(&definition.definition)?,
                    })
                })
                .collect::<Result<Vec<_>, Status>>()?,
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn get_all_schemas(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<rpc::schema_registry::Schemas>, Status> {
        let schemas = self.db.get_all_schemas().await?;

        Ok(Response::new(rpc::schema_registry::Schemas {
            schemas: schemas
                .into_iter()
                .map(|schema| rpc::schema_registry::Schema {
                    id: schema.id.to_string(),
                    metadata: rpc::schema_registry::SchemaMetadata {
                        name: schema.name,
                        insert_destination: schema.insert_destination,
                        query_address: schema.query_address,
                        r#type: schema.r#type.into(),
                    },
                })
                .collect(),
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn get_all_schemas_with_definitions(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<rpc::schema_registry::SchemasWithDefinitions>, Status> {
        let schemas = self.db.get_all_schemas_with_definitions().await?;

        Ok(Response::new(
            rpc::schema_registry::SchemasWithDefinitions {
                schemas: schemas
                    .into_iter()
                    .map(|schema| {
                        Ok(rpc::schema_registry::SchemaWithDefinitions {
                            id: schema.id.to_string(),
                            metadata: rpc::schema_registry::SchemaMetadata {
                                name: schema.name,
                                insert_destination: schema.insert_destination,
                                query_address: schema.query_address,
                                r#type: schema.r#type.into(),
                            },
                            definitions: schema
                                .definitions
                                .into_iter()
                                .map(|definition| {
                                    Ok(rpc::schema_registry::SchemaDefinition {
                                        version: definition.version.to_string(),
                                        definition: serialize_json(&definition.definition)?,
                                    })
                                })
                                .collect::<Result<Vec<_>, Status>>()?,
                        })
                    })
                    .collect::<Result<Vec<_>, Status>>()?,
            },
        ))
    }

    #[tracing::instrument(skip(self))]
    async fn validate_value(
        &self,
        request: Request<ValueToValidate>,
    ) -> Result<Response<Errors>, Status> {
        let request = request.into_inner();
        let versioned_id = VersionedUuid {
            id: parse_uuid(&request.schema_id.id)?,
            version_req: parse_optional_version_req(&request.schema_id.version_req)?
                .unwrap_or_else(VersionReq::any),
        };
        let json = parse_json_and_deserialize(&request.value)?;

        let (_version, definition) = self.db.get_schema_definition(&versioned_id).await?;
        let schema = jsonschema::JSONSchema::compile(&definition)
            .map_err(RegistryError::InvalidJsonSchema)?;
        let errors = match schema.validate(&json) {
            Ok(()) => vec![],
            Err(errors) => errors.map(|err| err.to_string()).collect(),
        };

        Ok(Response::new(Errors { errors }))
    }

    type WatchAllSchemaUpdatesStream = Pin<
        Box<
            dyn Stream<Item = Result<rpc::schema_registry::Schema, Status>> + Send + Sync + 'static,
        >,
    >;

    async fn watch_all_schema_updates(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::WatchAllSchemaUpdatesStream>, Status> {
        let schema_rx = self.db.listen_to_schema_updates().await?;

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::UnboundedReceiverStream::new(schema_rx).map(|schema| {
                let schema = schema?;

                Ok(rpc::schema_registry::Schema {
                    id: schema.id.to_string(),
                    metadata: rpc::schema_registry::SchemaMetadata {
                        name: schema.name,
                        insert_destination: schema.insert_destination,
                        query_address: schema.query_address,
                        r#type: schema.r#type.into(),
                    },
                })
            }),
        )))
    }
}

fn parse_optional_version_req(req: &Option<String>) -> Result<Option<VersionReq>, Status> {
    if let Some(req) = req.as_ref() {
        Ok(Some(VersionReq::parse(req).map_err(|err| {
            Status::invalid_argument(format!("Invalid version requirement provided: {}", err))
        })?))
    } else {
        Ok(None)
    }
}

fn parse_version(req: &str) -> Result<Version, Status> {
    Version::parse(req)
        .map_err(|err| Status::invalid_argument(format!("Invalid version provided: {}", err)))
}

fn parse_json_and_deserialize<T: serde::de::DeserializeOwned>(json: &[u8]) -> Result<T, Status> {
    serde_json::from_slice(json)
        .map_err(|err| Status::invalid_argument(format!("Invalid JSON provided: {}", err)))
}

fn parse_uuid(id: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(id)
        .map_err(|err| Status::invalid_argument(format!("Failed to parse UUID: {}", err)))
}

fn serialize_json<T: serde::Serialize>(json: &T) -> Result<Vec<u8>, Status> {
    serde_json::to_vec(json)
        .map_err(|err| Status::internal(format!("Unable to serialize JSON: {}", err)))
}
