use crate::{
    db::SchemaDb,
    error::RegistryError,
    replication::{KafkaConfig, ReplicationEvent, ReplicationRole, ReplicationState},
    types::DbExport,
    types::ViewUpdate,
    types::{NewSchema, NewSchemaVersion, VersionedUuid},
    View,
};
use anyhow::Context;
use indradb::SledDatastore;
use rpc::schema_registry::{
    schema_registry_server::SchemaRegistry, Empty, Errors, Id, NewSchemaView, PodName, Schema,
    SchemaMetadataUpdate, SchemaNames, SchemaQueryAddress, SchemaTopic, SchemaVersions,
    SchemaViews, Schemas, UpdatedView, ValueToValidate, VersionedId,
};
use semver::Version;
use semver::VersionReq;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tonic::{Request, Response, Status};
use utils::messaging_system::metadata_fetcher::KafkaMetadataFetcher;
use utils::{abort_on_poison, messaging_system::Result};
use uuid::Uuid;

pub struct SchemaRegistryImpl {
    pub db: Arc<SchemaDb>,
    pub mq_metadata: Arc<KafkaMetadataFetcher>,
    pub replication: Arc<Mutex<ReplicationState>>,
    pub pod_name: Option<String>,
}

impl SchemaRegistryImpl {
    pub async fn new(
        db: SledDatastore,
        replication_role: ReplicationRole,
        kafka_config: KafkaConfig,
        pod_name: Option<String>,
    ) -> Result<Self> {
        let child_db = Arc::new(SchemaDb { db });
        let mq_metadata = Arc::new(KafkaMetadataFetcher::new(&kafka_config.brokers).await?);
        let schema_registry = Self {
            db: child_db.clone(),
            mq_metadata,
            replication: Arc::new(Mutex::new(ReplicationState::new(kafka_config, child_db))),
            pod_name,
        };
        schema_registry.set_replication_role(replication_role);
        Ok(schema_registry)
    }

    pub fn set_replication_role(&self, role: ReplicationRole) {
        self.replication
            .lock()
            .unwrap_or_else(abort_on_poison)
            .set_role(role);
    }

    fn replicate_message(&self, event: ReplicationEvent) {
        self.replication
            .lock()
            .unwrap_or_else(abort_on_poison)
            .replicate_message(event)
    }

    pub fn export_all(&self) -> anyhow::Result<DbExport> {
        self.db
            .export_all()
            .context("Failed to export the entire database")
    }

    pub fn import_all(&self, imported: DbExport) -> anyhow::Result<()> {
        self.db
            .import_all(imported)
            .context("failed to import database")
    }
}

#[tonic::async_trait]
impl SchemaRegistry for SchemaRegistryImpl {
    async fn add_schema(
        &self,
        request: Request<rpc::schema_registry::NewSchema>,
    ) -> Result<Response<Id>, Status> {
        let request = request.into_inner();
        let schema_id = parse_optional_uuid(&request.id)?;
        let schema_type = request.schema_type().into();

        let new_schema = NewSchema {
            name: request.name,
            definition: parse_json(&request.definition)?,
            query_address: request.query_address,
            kafka_topic: request.topic,
            schema_type,
        };

        if !self
            .mq_metadata
            .topic_exists(&new_schema.kafka_topic)
            .await
            .map_err(RegistryError::from)?
        {
            return Err(RegistryError::NoTopic(new_schema.kafka_topic).into());
        }

        let new_id = self.db.add_schema(new_schema.clone(), schema_id)?;
        self.replicate_message(ReplicationEvent::AddSchema {
            id: new_id,
            schema: new_schema,
        });

        Ok(Response::new(Id {
            id: new_id.to_string(),
        }))
    }

    async fn add_schema_version(
        &self,
        request: Request<rpc::schema_registry::NewSchemaVersion>,
    ) -> Result<Response<Empty>, Status> {
        let request = request.into_inner();
        let schema_id = parse_uuid(&request.id)?;
        let new_version = NewSchemaVersion {
            version: parse_version(&request.version)?,
            definition: parse_json(&request.definition)?,
        };

        self.db
            .add_new_version_of_schema(schema_id, new_version.clone())?;
        self.replicate_message(ReplicationEvent::AddSchemaVersion {
            id: schema_id,
            new_version,
        });

        Ok(Response::new(Empty {}))
    }

    async fn update_schema_metadata(
        &self,
        request: Request<SchemaMetadataUpdate>,
    ) -> Result<Response<Empty>, Status> {
        let request = request.into_inner();
        let schema_id = parse_uuid(&request.id)?;

        if let Some(topic) = request.topic.as_ref() {
            if !self
                .mq_metadata
                .topic_exists(&topic)
                .await
                .map_err(RegistryError::from)?
            {
                return Err(RegistryError::NoTopic(topic.clone()).into());
            }
        }

        if let Some(name) = request.name.as_ref() {
            self.db.update_schema_name(schema_id, name.clone())?;
        }
        if let Some(topic) = request.topic.as_ref() {
            self.db.update_schema_topic(schema_id, topic.clone())?;
        }
        if let Some(address) = request.address.as_ref() {
            self.db
                .update_schema_query_address(schema_id, address.clone())?;
        }

        // Previously we were using method `schema_type()` but it has implicit conversion which may hit us in the bottom:
        // `from_i32().unwrap_or_default()`
        // Therefore its better to return an error when we expect `1` or `2` but we get `3` than assume that `3` is `1`.
        if let Some(schema_type) = request.schema_type {
            let schema_type = rpc::schema_registry::schema_type::Type::from_i32(schema_type)
                .ok_or(RegistryError::InvalidSchemaType)?;
            self.db.update_schema_type(schema_id, schema_type.into())?;
        }

        self.replicate_message(ReplicationEvent::UpdateSchemaMetadata {
            id: schema_id,
            name: request.name,
            topic: request.topic,
            query_address: request.address,
            schema_type: None,
        });

        Ok(Response::new(Empty {}))
    }

    async fn add_view_to_schema(
        &self,
        request: Request<NewSchemaView>,
    ) -> Result<Response<Id>, Status> {
        let request = request.into_inner();
        let schema_id = parse_uuid(&request.schema_id)?;
        let replicated_view_id = parse_optional_uuid(&request.view_id)?;
        let view = View {
            name: request.name,
            jmespath: request.jmespath,
        };

        let view_id = self
            .db
            .add_view_to_schema(schema_id, view.clone(), replicated_view_id)?;
        self.replicate_message(ReplicationEvent::AddViewToSchema {
            schema_id,
            view,
            view_id,
        });

        Ok(Response::new(Id {
            id: view_id.to_string(),
        }))
    }

    async fn update_view(
        &self,
        request: Request<UpdatedView>,
    ) -> Result<Response<rpc::schema_registry::View>, Status> {
        let request = request.into_inner();
        let view_id = parse_uuid(&request.id)?;
        let view = ViewUpdate {
            name: request.name,
            jmespath: request.jmespath,
        };

        let old_view = self.db.update_view(view_id, view.clone())?;
        self.replicate_message(ReplicationEvent::UpdateView { id: view_id, view });

        Ok(Response::new(rpc::schema_registry::View {
            name: old_view.name,
            jmespath: old_view.jmespath,
        }))
    }

    async fn get_schema(
        &self,
        request: Request<VersionedId>,
    ) -> Result<Response<rpc::schema_registry::SchemaDefinition>, Status> {
        let request = request.into_inner();
        let id = VersionedUuid::new(
            parse_uuid(&request.id)?,
            parse_version_req(&request.version_req)?,
        );
        let definition = self.db.get_schema_definition(&id)?;

        Ok(Response::new(rpc::schema_registry::SchemaDefinition {
            version: definition.version.to_string(),
            definition: serialize_json(&definition.definition)?,
        }))
    }

    async fn get_schema_metadata(&self, request: Request<Id>) -> Result<Response<Schema>, Status> {
        let request = request.into_inner();
        let schema_id = parse_uuid(&request.id)?;
        let schema = self.db.get_schema(schema_id)?;

        let schema = Schema {
            name: schema.name,
            topic: schema.kafka_topic,
            query_address: schema.query_address,
            schema_type: schema.schema_type as i32,
        };

        Ok(Response::new(schema))
    }

    async fn get_schema_topic(
        &self,
        request: Request<Id>,
    ) -> Result<Response<SchemaTopic>, Status> {
        let request = request.into_inner();
        let schema_id = parse_uuid(&request.id)?;
        let topic = self.db.get_schema_topic(schema_id)?;

        Ok(Response::new(SchemaTopic { topic }))
    }

    async fn get_schema_query_address(
        &self,
        request: Request<Id>,
    ) -> Result<Response<SchemaQueryAddress>, Status> {
        let request = request.into_inner();
        let schema_id = parse_uuid(&request.id)?;
        let address = self.db.get_schema_query_address(schema_id)?;

        Ok(Response::new(SchemaQueryAddress { address }))
    }

    async fn get_schema_versions(
        &self,
        request: Request<Id>,
    ) -> Result<Response<SchemaVersions>, Status> {
        let schema_id = parse_uuid(&request.into_inner().id)?;
        let all_versions = self.db.get_schema_versions(schema_id)?;
        let versions = all_versions
            .into_iter()
            .map(|(version, _vertex_id)| version.to_string())
            .collect();

        Ok(Response::new(SchemaVersions { versions }))
    }

    async fn get_schema_type(
        &self,
        request: Request<Id>,
    ) -> Result<Response<rpc::schema_registry::SchemaType>, Status> {
        let request = request.into_inner();
        let schema_id = parse_uuid(&request.id)?;
        let schema_type = self.db.get_schema_type(schema_id)? as i32;

        Ok(Response::new(rpc::schema_registry::SchemaType {
            schema_type,
        }))
    }

    async fn get_view(
        &self,
        request: Request<Id>,
    ) -> Result<Response<rpc::schema_registry::View>, Status> {
        let view_id = parse_uuid(&request.into_inner().id)?;
        let view = self.db.get_view(view_id)?;

        Ok(Response::new(rpc::schema_registry::View {
            name: view.name,
            jmespath: view.jmespath,
        }))
    }

    async fn get_all_schemas(&self, _request: Request<Empty>) -> Result<Response<Schemas>, Status> {
        let schemas = self.db.get_all_schemas()?;

        Ok(Response::new(Schemas {
            schemas: schemas
                .into_iter()
                .map(|(schema_id, schema)| {
                    (
                        schema_id.to_string(),
                        Schema {
                            name: schema.name,
                            topic: schema.kafka_topic,
                            query_address: schema.query_address,
                            schema_type: schema.schema_type as i32,
                        },
                    )
                })
                .collect(),
        }))
    }

    async fn get_all_schema_names(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<SchemaNames>, Status> {
        let names = self.db.get_all_schema_names()?;

        Ok(Response::new(SchemaNames {
            names: names
                .into_iter()
                .map(|(schema_id, name)| (schema_id.to_string(), name))
                .collect(),
        }))
    }

    async fn get_all_views_of_schema(
        &self,
        request: Request<Id>,
    ) -> Result<Response<SchemaViews>, Status> {
        let schema_id = parse_uuid(&request.into_inner().id)?;
        let views = self.db.get_all_views_of_schema(schema_id)?;

        Ok(Response::new(SchemaViews {
            views: views
                .into_iter()
                .map(|(view_id, view)| {
                    (
                        view_id.to_string(),
                        rpc::schema_registry::View {
                            name: view.name,
                            jmespath: view.jmespath,
                        },
                    )
                })
                .collect(),
        }))
    }

    async fn validate_value(
        &self,
        request: Request<ValueToValidate>,
    ) -> Result<Response<Errors>, Status> {
        let request = request.into_inner();
        let schema_id = parse_uuid(&request.schema_id)?;
        let json = parse_json(&request.value)?;

        let full_schema = self
            .db
            .get_schema_definition(&VersionedUuid::any(schema_id))?;
        let schema = jsonschema::JSONSchema::compile(&full_schema.definition)
            .map_err(|_err| RegistryError::InvalidSchema)?;
        let errors = match schema.validate(&json) {
            Ok(()) => vec![],
            Err(errors) => errors.map(|err| err.to_string()).collect(),
        };

        Ok(Response::new(Errors { errors }))
    }

    async fn promote_to_master(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<PodName>, Status> {
        self.replication
            .lock()
            .unwrap_or_else(abort_on_poison)
            .set_role(ReplicationRole::Master);

        Ok(Response::new(PodName {
            name: self.pod_name.clone().unwrap_or_default(),
        }))
    }

    async fn heartbeat(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        //empty
        Ok(Response::new(Empty {}))
    }
}

fn parse_version_req(req: &str) -> Result<VersionReq, Status> {
    VersionReq::parse(req).map_err(|err| {
        Status::invalid_argument(format!("Invalid version requirement provided: {}", err))
    })
}

fn parse_version(req: &str) -> Result<Version, Status> {
    Version::parse(req)
        .map_err(|err| Status::invalid_argument(format!("Invalid version provided: {}", err)))
}

fn parse_json(json: &str) -> Result<Value, Status> {
    serde_json::from_str(json)
        .map_err(|err| Status::invalid_argument(format!("Invalid JSON provided: {}", err)))
}

fn parse_uuid(id: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(id)
        .map_err(|err| Status::invalid_argument(format!("Failed to parse UUID: {}", err)))
}

fn parse_optional_uuid(id: &str) -> Result<Option<Uuid>, Status> {
    if id.is_empty() {
        Ok(None)
    } else {
        Ok(Some(Uuid::parse_str(id).map_err(|err| {
            Status::invalid_argument(format!("Failed to parse optional UUID: {}", err))
        })?))
    }
}

fn serialize_json(json: &Value) -> Result<String, Status> {
    serde_json::to_string(json)
        .map_err(|err| Status::internal(format!("Unable to serialize JSON: {}", err)))
}
