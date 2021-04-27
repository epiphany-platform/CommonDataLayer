use std::collections::HashMap;
use std::convert::TryInto;
use std::pin::Pin;

use semver::Version;
use semver::VersionReq;
use sqlx::types::Json;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::config::CommunicationMethodConfig;
use crate::config::Config;
use crate::db::SchemaRegistryDb;
use crate::error::{RegistryError, RegistryResult};
use crate::types::schema::{NewSchema, SchemaDefinition, SchemaUpdate};
use crate::types::view::{NewView, ViewUpdate};
use crate::types::VersionedUuid;
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
        config: &Config,
        communication_config: CommunicationMethodConfig,
    ) -> anyhow::Result<Self> {
        let db = SchemaRegistryDb::new(config).await?;
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
}

#[tonic::async_trait]
impl SchemaRegistry for SchemaRegistryImpl {
    #[tracing::instrument(skip(self))]
    async fn add_schema(
        &self,
        request: Request<rpc::schema_registry::NewSchema>,
    ) -> Result<Response<Id>, Status> {
        utils::tracing::grpc::set_parent_span(&request);
        let request = request.into_inner();
        let new_schema = NewSchema {
            name: request.metadata.name,
            definition: parse_json_and_deserialize(&request.definition)?,
            query_address: request.metadata.query_address,
            insert_destination: request.metadata.insert_destination,
            schema_type: request.metadata.schema_type.try_into()?,
        };

        if !new_schema.insert_destination.is_empty()
            && !self
                .mq_metadata
                .destination_exists(&new_schema.insert_destination)
                .await
                .map_err(RegistryError::from)?
        {
            return Err(
                RegistryError::NoInsertDestination(new_schema.insert_destination.clone()).into(),
            );
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
        utils::tracing::grpc::set_parent_span(&request);
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
        utils::tracing::grpc::set_parent_span(&request);
        let request = request.into_inner();
        let schema_id = parse_uuid(&request.id)?;

        let schema_type = if let Some(st) = request.patch.schema_type {
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
                return Err(RegistryError::NoInsertDestination(destination.clone()).into());
            }
        }

        self.db
            .update_schema(
                schema_id,
                SchemaUpdate {
                    name: request.patch.name,
                    query_address: request.patch.query_address,
                    insert_destination: request.patch.insert_destination,
                    schema_type,
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
        utils::tracing::grpc::set_parent_span(&request);
        let request = request.into_inner();
        let id = parse_uuid(&request.id)?;

        let schema = self.db.get_schema(id).await?;

        Ok(Response::new(rpc::schema_registry::SchemaMetadata {
            name: schema.name,
            insert_destination: schema.insert_destination,
            query_address: schema.query_address,
            schema_type: schema.schema_type.into(),
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn get_base_schema_of_view(
        &self,
        request: Request<Id>,
    ) -> Result<Response<rpc::schema_registry::Schema>, Status> {
        utils::tracing::grpc::set_parent_span(&request);
        let request = request.into_inner();
        let id = parse_uuid(&request.id)?;

        let schema = self.db.get_base_schema_of_view(id).await?;

        Ok(Response::new(rpc::schema_registry::Schema {
            id: schema.id.to_string(),
            metadata: rpc::schema_registry::SchemaMetadata {
                name: schema.name,
                insert_destination: schema.insert_destination,
                query_address: schema.query_address,
                schema_type: schema.schema_type.into(),
            },
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn get_schema_definition(
        &self,
        request: Request<VersionedId>,
    ) -> Result<Response<rpc::schema_registry::SchemaDefinition>, Status> {
        utils::tracing::grpc::set_parent_span(&request);
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
        utils::tracing::grpc::set_parent_span(&request);
        let request = request.into_inner();
        let id = parse_uuid(&request.id)?;

        let versions = self.db.get_schema_versions(id).await?;

        Ok(Response::new(rpc::schema_registry::SchemaVersions {
            versions: versions.into_iter().map(|v| v.to_string()).collect(),
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn get_full_schema(
        &self,
        request: Request<Id>,
    ) -> Result<Response<rpc::schema_registry::FullSchema>, Status> {
        utils::tracing::grpc::set_parent_span(&request);
        let request = request.into_inner();
        let id = parse_uuid(&request.id)?;

        let schema = self.db.get_full_schema(id).await?;

        Ok(Response::new(rpc::schema_registry::FullSchema {
            id: request.id,
            metadata: rpc::schema_registry::SchemaMetadata {
                name: schema.name,
                insert_destination: schema.insert_destination,
                query_address: schema.query_address,
                schema_type: schema.schema_type.into(),
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
            views: schema
                .views
                .into_iter()
                .map(|view| {
                    Ok(rpc::schema_registry::View {
                        id: view.id.to_string(),
                        name: view.name,
                        materializer_address: view.materializer_address,
                        materializer_options: serde_json::to_string(&view.materializer_options)
                            .map_err(RegistryError::MalformedViewFields)?,
                        fields: view
                            .fields
                            .0
                            .into_iter()
                            .map(|(name, value)| {
                                Ok((
                                    name,
                                    serde_json::to_string(&value)
                                        .map_err(RegistryError::MalformedViewFields)?,
                                ))
                            })
                            .collect::<RegistryResult<HashMap<_, _>>>()?,
                    })
                })
                .collect::<RegistryResult<Vec<_>>>()?,
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn get_all_schemas(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<rpc::schema_registry::Schemas>, Status> {
        utils::tracing::grpc::set_parent_span(&request);
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
                        schema_type: schema.schema_type.into(),
                    },
                })
                .collect(),
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn get_all_full_schemas(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<rpc::schema_registry::FullSchemas>, Status> {
        utils::tracing::grpc::set_parent_span(&request);
        let schemas = self.db.get_all_full_schemas().await?;

        Ok(Response::new(rpc::schema_registry::FullSchemas {
            schemas: schemas
                .into_iter()
                .map(|schema| {
                    Ok(rpc::schema_registry::FullSchema {
                        id: schema.id.to_string(),
                        metadata: rpc::schema_registry::SchemaMetadata {
                            name: schema.name,
                            insert_destination: schema.insert_destination,
                            query_address: schema.query_address,
                            schema_type: schema.schema_type.into(),
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
                        views: schema
                            .views
                            .into_iter()
                            .map(|view| {
                                Ok(rpc::schema_registry::View {
                                    id: view.id.to_string(),
                                    name: view.name,
                                    materializer_address: view.materializer_address,
                                    materializer_options: serde_json::to_string(
                                        &view.materializer_options,
                                    )
                                    .map_err(RegistryError::MalformedViewFields)?,
                                    fields: view
                                        .fields
                                        .0
                                        .into_iter()
                                        .map(|(name, value)| {
                                            Ok((
                                                name,
                                                serde_json::to_string(&value)
                                                    .map_err(RegistryError::MalformedViewFields)?,
                                            ))
                                        })
                                        .collect::<RegistryResult<HashMap<_, _>>>()?,
                                })
                            })
                            .collect::<RegistryResult<Vec<_>>>()?,
                    })
                })
                .collect::<Result<Vec<_>, Status>>()?,
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn get_view(
        &self,
        request: Request<Id>,
    ) -> Result<Response<rpc::schema_registry::View>, Status> {
        utils::tracing::grpc::set_parent_span(&request);
        let request = request.into_inner();
        let id = parse_uuid(&request.id)?;

        let view = self.db.get_view(id).await?;

        Ok(Response::new(rpc::schema_registry::View {
            id: request.id,
            name: view.name,
            materializer_address: view.materializer_address,
            materializer_options: serde_json::to_string(&view.materializer_options)
                .map_err(RegistryError::MalformedViewFields)?,
            fields: view
                .fields
                .0
                .into_iter()
                .map(|(key, value)| {
                    Ok((
                        key,
                        serde_json::to_string(&value)
                            .map_err(RegistryError::MalformedViewFields)?,
                    ))
                })
                .collect::<RegistryResult<_>>()?,
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn get_all_views_of_schema(
        &self,
        request: Request<Id>,
    ) -> Result<Response<rpc::schema_registry::SchemaViews>, Status> {
        utils::tracing::grpc::set_parent_span(&request);
        let request = request.into_inner();
        let schema_id = parse_uuid(&request.id)?;

        let views = self.db.get_all_views_of_schema(schema_id).await?;

        Ok(Response::new(rpc::schema_registry::SchemaViews {
            views: views
                .into_iter()
                .map(|view| {
                    Ok(rpc::schema_registry::View {
                        id: view.id.to_string(),
                        name: view.name,
                        materializer_address: view.materializer_address,
                        materializer_options: serde_json::to_string(&view.materializer_options)
                            .map_err(RegistryError::MalformedViewFields)?,
                        fields: view
                            .fields
                            .0
                            .into_iter()
                            .map(|(key, value)| {
                                Ok((
                                    key,
                                    serde_json::to_string(&value)
                                        .map_err(RegistryError::MalformedViewFields)?,
                                ))
                            })
                            .collect::<RegistryResult<_>>()?,
                    })
                })
                .collect::<RegistryResult<Vec<_>>>()?,
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn add_view_to_schema(
        &self,
        request: Request<rpc::schema_registry::NewView>,
    ) -> Result<Response<Id>, Status> {
        utils::tracing::grpc::set_parent_span(&request);
        //TODO: Request materializer validation for the options
        let request = request.into_inner();
        let new_view = NewView {
            schema_id: parse_uuid(&request.schema_id)?,
            name: request.name,
            materializer_address: request.materializer_address,
            materializer_options: serde_json::from_str(&request.materializer_options)
                .map_err(RegistryError::MalformedViewFields)?,
            fields: Json(
                request
                    .fields
                    .into_iter()
                    .map(|(key, value)| {
                        Ok((
                            key,
                            serde_json::from_str(&value)
                                .map_err(RegistryError::MalformedViewFields)?,
                        ))
                    })
                    .collect::<RegistryResult<HashMap<_, _>>>()?,
            ),
        };

        let new_id = self.db.add_view_to_schema(new_view).await?;

        Ok(Response::new(Id {
            id: new_id.to_string(),
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn update_view(
        &self,
        request: Request<rpc::schema_registry::ViewUpdate>,
    ) -> Result<Response<Empty>, Status> {
        utils::tracing::grpc::set_parent_span(&request);
        let request = request.into_inner();
        let id = parse_uuid(&request.id)?;

        //TODO: Request materializer validation for the options
        let fields = if request.update_fields {
            Some(Json(
                request
                    .fields
                    .into_iter()
                    .map(|(key, value)| {
                        Ok((
                            key,
                            serde_json::from_str(&value)
                                .map_err(RegistryError::MalformedViewFields)?,
                        ))
                    })
                    .collect::<RegistryResult<HashMap<_, _>>>()?,
            ))
        } else {
            None
        };
        let update = ViewUpdate {
            name: request.name,
            materializer_address: request.materializer_address,
            materializer_options: if !request.materializer_options.is_empty() {
                Some(
                    serde_json::from_str(&request.materializer_options)
                        .map_err(RegistryError::MalformedViewFields)?,
                )
            } else {
                None
            },
            fields,
        };

        self.db.update_view(id, update).await?;

        Ok(Response::new(Empty {}))
    }

    async fn validate_value(
        &self,
        request: Request<ValueToValidate>,
    ) -> Result<Response<Errors>, Status> {
        utils::tracing::grpc::set_parent_span(&request);
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
        request: Request<Empty>,
    ) -> Result<Response<Self::WatchAllSchemaUpdatesStream>, Status> {
        utils::tracing::grpc::set_parent_span(&request);
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
                        schema_type: schema.schema_type.into(),
                    },
                })
            }),
        )))
    }

    async fn ping(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        utils::tracing::grpc::set_parent_span(&request);
        Ok(Response::new(Empty {}))
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
