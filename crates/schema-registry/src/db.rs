use std::collections::HashMap;

use semver::Version;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::postgres::{PgConnectOptions, PgListener, PgPool, PgPoolOptions};
use sqlx::types::Json;
use sqlx::{Acquire, Connection, Postgres};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tracing::{trace, warn};
use uuid::Uuid;

use crate::error::{RegistryError, RegistryResult};
use crate::settings::Settings;
use crate::types::schema::{FullSchema, NewSchema, Schema, SchemaDefinition, SchemaUpdate};
use crate::types::view::{NewView, View, ViewUpdate};
use crate::types::DbExport;
use crate::types::VersionedUuid;
use crate::utils::build_full_schema;
use utils::types::materialization::FieldDefinition;

const SCHEMAS_LISTEN_CHANNEL: &str = "schemas";
const VIEWS_LISTEN_CHANNEL: &str = "views";

pub struct SchemaRegistryDb {
    pool: PgPool,
    db_schema: String,
}

impl SchemaRegistryDb {
    pub async fn new(config: &Settings) -> RegistryResult<Self> {
        let options = PgConnectOptions::new()
            .host(&config.postgres.host)
            .port(config.postgres.port)
            .username(&config.postgres.username)
            .password(&config.postgres.password)
            .database(&config.postgres.dbname);

        Ok(Self {
            pool: PgPoolOptions::new()
                .connect_with(options)
                .await
                .map_err(RegistryError::ConnectionError)?,
            db_schema: config.postgres.schema.clone(),
        })
    }

    async fn connect(&self) -> RegistryResult<PoolConnection<Postgres>> {
        let mut conn = self.pool.acquire().await?;
        self.set_schema_for_connection(&mut conn).await?;

        Ok(conn)
    }

    // TODO: remove need to set schema on each connection
    async fn set_schema_for_connection(
        &self,
        conn: &mut PoolConnection<Postgres>,
    ) -> RegistryResult<()> {
        sqlx::query(&format!("SET SCHEMA '{}'", &self.db_schema))
            .execute(conn)
            .await?;

        Ok(())
    }

    pub async fn get_schema(&self, id: Uuid) -> RegistryResult<Schema> {
        let mut conn = self.connect().await?;

        sqlx::query_as!(
            Schema,
            "SELECT id, name, insert_destination, query_address, schema_type as \"schema_type: _\"
             FROM schemas WHERE id = $1",
            id
        )
        .fetch_optional(&mut conn)
        .await?
        .ok_or(RegistryError::NoSchemaWithId(id))
    }

    pub async fn get_base_schema_of_view(&self, id: Uuid) -> RegistryResult<Schema> {
        let mut conn = self.connect().await?;

        sqlx::query_as!(
            Schema,
            "SELECT id, name, insert_destination, query_address, schema_type as \"schema_type: _\"
             FROM schemas WHERE id = (SELECT schema FROM views WHERE id = $1)",
            id
        )
        .fetch_one(&mut conn)
        .await
        .map_err(RegistryError::DbError)
    }

    pub async fn get_full_schema(&self, id: Uuid) -> RegistryResult<FullSchema> {
        let mut conn = self.connect().await?;
        let schema = self.get_schema(id).await?;

        let definitions = sqlx::query!(
            "SELECT version, definition FROM definitions WHERE schema = $1",
            id
        )
        .fetch_all(&mut conn)
        .await?
        .into_iter()
        .map(|row| {
            Ok(SchemaDefinition {
                version: Version::parse(&row.version).map_err(RegistryError::InvalidVersion)?,
                definition: row.definition,
            })
        })
        .collect::<RegistryResult<Vec<SchemaDefinition>>>()?;

        let views = sqlx::query_as!(
            View,
            "SELECT id, name, materializer_address, materializer_options, fields as \"fields: _\"
             FROM views WHERE schema = $1",
            id
        )
        .fetch_all(&mut conn)
        .await?;

        Ok(FullSchema {
            id: schema.id,
            name: schema.name,
            schema_type: schema.schema_type,
            insert_destination: schema.insert_destination,
            query_address: schema.query_address,
            definitions,
            views,
        })
    }

    pub async fn get_schema_definition(
        &self,
        id: &VersionedUuid,
    ) -> RegistryResult<(Version, Value)> {
        let mut conn = self.connect().await?;

        let version = self.get_latest_valid_schema_version(id).await?;
        let row = sqlx::query!(
            "SELECT definition FROM definitions WHERE schema = $1 and version = $2",
            id.id,
            version.to_string()
        )
        .fetch_one(&mut conn)
        .await?;

        Ok((version, row.definition))
    }

    pub async fn get_view(&self, id: Uuid) -> RegistryResult<View> {
        let mut conn = self.connect().await?;

        sqlx::query_as!(
            View,
            "SELECT id, name, materializer_address, materializer_options, fields as \"fields: _\"
             FROM views WHERE id = $1",
            id
        )
        .fetch_optional(&mut conn)
        .await?
        .ok_or(RegistryError::NoViewWithId(id))
    }

    pub async fn get_all_views_of_schema(&self, schema_id: Uuid) -> RegistryResult<Vec<View>> {
        let mut conn = self.connect().await?;

        sqlx::query_as!(
            View,
            "SELECT id, name, materializer_address, materializer_options, fields as \"fields: _\"
             FROM views WHERE schema = $1",
            schema_id
        )
        .fetch_all(&mut conn)
        .await
        .map_err(RegistryError::DbError)
    }

    pub async fn get_schema_versions(&self, id: Uuid) -> RegistryResult<Vec<Version>> {
        let mut conn = self.connect().await?;

        sqlx::query!("SELECT version FROM definitions WHERE schema = $1", id)
            .fetch_all(&mut conn)
            .await
            .map_err(RegistryError::DbError)?
            .into_iter()
            .map(|row| Version::parse(&row.version).map_err(RegistryError::InvalidVersion))
            .collect()
    }

    async fn get_latest_valid_schema_version(&self, id: &VersionedUuid) -> RegistryResult<Version> {
        self.get_schema_versions(id.id)
            .await?
            .into_iter()
            .filter(|version| id.version_req.matches(version))
            .max()
            .ok_or_else(|| RegistryError::NoVersionMatchesRequirement(id.clone()))
    }

    pub async fn get_all_schemas(&self) -> RegistryResult<Vec<Schema>> {
        let mut conn = self.connect().await?;

        sqlx::query_as!(
            Schema,
            "SELECT id, name, insert_destination, query_address, schema_type as \"schema_type: _\" \
             FROM schemas ORDER BY name"
        )
        .fetch_all(&mut conn)
        .await
        .map_err(RegistryError::DbError)
    }

    pub async fn get_all_full_schemas(&self) -> RegistryResult<Vec<FullSchema>> {
        let mut conn = self.connect().await?;

        let all_schemas = sqlx::query_as!(
            Schema,
            "SELECT id, name, insert_destination, query_address, schema_type as \"schema_type: _\" FROM schemas"
        )
        .fetch_all(&mut conn)
        .await?;
        let mut all_definitions =
            sqlx::query!("SELECT version, definition, schema FROM definitions")
                .fetch_all(&mut conn)
                .await?;
        let mut all_views =
            sqlx::query!("SELECT id, name, materializer_address, materializer_options, fields, schema FROM views",)
                .fetch_all(&mut conn)
                .await?;

        all_schemas
            .into_iter()
            .map(|schema: Schema| {
                let definitions = all_definitions
                    .drain_filter(|d| d.schema == schema.id)
                    .map(|row| {
                        Ok(SchemaDefinition {
                            version: Version::parse(&row.version)
                                .map_err(RegistryError::InvalidVersion)?,
                            definition: row.definition,
                        })
                    })
                    .collect::<RegistryResult<Vec<SchemaDefinition>>>()?;
                let views = all_views
                    .drain_filter(|v| v.schema == schema.id)
                    .map(|row| {
                        Ok(View {
                            id: row.id,
                            name: row.name,
                            materializer_address: row.materializer_address,
                            materializer_options: row.materializer_options,
                            fields:
                                serde_json::from_value::<Json<HashMap<String, FieldDefinition>>>(
                                    row.fields,
                                )
                                .map_err(RegistryError::MalformedViewFields)?,
                        })
                    })
                    .collect::<RegistryResult<Vec<View>>>()?;

                Ok(FullSchema {
                    id: schema.id,
                    name: schema.name,
                    schema_type: schema.schema_type,
                    insert_destination: schema.insert_destination,
                    query_address: schema.query_address,
                    definitions,
                    views,
                })
            })
            .collect()
    }

    pub async fn add_schema(&self, mut schema: NewSchema) -> RegistryResult<Uuid> {
        let mut conn = self.connect().await?;

        let new_id = Uuid::new_v4();
        build_full_schema(&mut schema.definition, self).await?;

        conn
            .transaction::<_, _, RegistryError>(move |c| {
                Box::pin(async move {
                    sqlx::query!(
                        "INSERT INTO schemas(id, name, schema_type, insert_destination, query_address) \
                         VALUES($1, $2, $3, $4, $5)",
                        &new_id,
                        &schema.name,
                        &schema.schema_type as &rpc::schema_registry::types::SchemaType,
                        &schema.insert_destination,
                        &schema.query_address,
                    )
                    .execute(c.acquire().await?)
                    .await?;

                    sqlx::query!(
                        "INSERT INTO definitions(version, definition, schema) \
                         VALUES('1.0.0', $1, $2)",
                        schema.definition,
                        new_id
                    )
                    .execute(c)
                    .await?;

                    Ok(())
                })
            })
            .await?;

        trace!("Add schema {}", new_id);

        Ok(new_id)
    }

    pub async fn add_view_to_schema(&self, new_view: NewView) -> RegistryResult<Uuid> {
        let mut conn = self.connect().await?;
        let new_id = Uuid::new_v4();

        sqlx::query!(
            "INSERT INTO views(id, schema, name, materializer_address, fields)
             VALUES ($1, $2, $3, $4, $5)",
            new_id,
            new_view.schema_id,
            new_view.name,
            new_view.materializer_address,
            serde_json::to_value(&new_view.fields).map_err(RegistryError::MalformedViewFields)?,
        )
        .execute(&mut conn)
        .await?;

        Ok(new_id)
    }

    pub async fn update_schema(&self, id: Uuid, update: SchemaUpdate) -> RegistryResult<()> {
        let mut conn = self.connect().await?;
        let old_schema = self.get_schema(id).await?;

        sqlx::query!(
            "UPDATE schemas SET name = $1, schema_type = $2, insert_destination = $3, query_address = $4 WHERE id = $5",
            update.name.unwrap_or(old_schema.name),
            update.schema_type.unwrap_or(old_schema.schema_type) as _,
            update.insert_destination.unwrap_or(old_schema.insert_destination),
            update.query_address.unwrap_or(old_schema.query_address),
            id
        )
        .execute(&mut conn)
        .await?;

        Ok(())
    }

    pub async fn update_view(&self, id: Uuid, update: ViewUpdate) -> RegistryResult<()> {
        let mut conn = self.connect().await?;
        let old = self.get_view(id).await?;

        sqlx::query!(
            "UPDATE views SET name = $1, materializer_address = $2, fields = $3
             WHERE id = $4",
            update.name.unwrap_or(old.name),
            update
                .materializer_address
                .unwrap_or(old.materializer_address),
            serde_json::to_value(&update.fields.unwrap_or(old.fields))
                .map_err(RegistryError::MalformedViewFields)?,
            id,
        )
        .execute(&mut conn)
        .await?;

        Ok(())
    }

    pub async fn add_new_version_of_schema(
        &self,
        id: Uuid,
        new_version: SchemaDefinition,
    ) -> RegistryResult<()> {
        let mut conn = self.connect().await?;
        self.get_schema(id).await?;

        if let Some(max_version) = self.get_schema_versions(id).await?.into_iter().max() {
            if max_version >= new_version.version {
                return Err(RegistryError::NewVersionMustBeGreatest {
                    schema_id: id,
                    max_version,
                });
            }
        }

        sqlx::query!(
            "INSERT INTO definitions(version, definition, schema) VALUES($1, $2, $3)",
            new_version.version.to_string(),
            new_version.definition,
            id
        )
        .execute(&mut conn)
        .await?;

        Ok(())
    }

    pub async fn validate_data_with_schema(
        &self,
        schema_id: VersionedUuid,
        json: &Value,
    ) -> RegistryResult<()> {
        let (_version, definition) = self.get_schema_definition(&schema_id).await?;
        let schema = jsonschema::JSONSchema::compile(&definition)
            .map_err(RegistryError::InvalidJsonSchema)?;

        let result = match schema.validate(&json) {
            Ok(()) => Ok(()),
            Err(errors) => Err(RegistryError::InvalidData(
                errors.map(|err| err.to_string()).collect(),
            )),
        };

        result
    }

    pub async fn listen_to_schema_updates(
        &self,
    ) -> RegistryResult<UnboundedReceiver<RegistryResult<Schema>>> {
        let (tx, rx) = unbounded_channel::<RegistryResult<Schema>>();
        let mut listener = PgListener::connect_with(&self.pool)
            .await
            .map_err(RegistryError::ConnectionError)?;
        listener.listen(SCHEMAS_LISTEN_CHANNEL).await?;

        tokio::spawn(async move {
            loop {
                let notification = listener
                    .recv()
                    .await
                    .map_err(RegistryError::NotificationError);
                let schema = notification.and_then(|n| {
                    serde_json::from_str::<Schema>(n.payload())
                        .map_err(RegistryError::MalformedNotification)
                });

                if tx.send(schema).is_err() {
                    return;
                }
            }
        });

        Ok(rx)
    }

    pub async fn listen_to_view_updates(
        &self,
    ) -> RegistryResult<UnboundedReceiver<RegistryResult<View>>> {
        let (tx, rx) = unbounded_channel::<RegistryResult<View>>();
        let mut listener = PgListener::connect_with(&self.pool)
            .await
            .map_err(RegistryError::ConnectionError)?;
        listener.listen(VIEWS_LISTEN_CHANNEL).await?;

        tokio::spawn(async move {
            loop {
                let notification = listener
                    .recv()
                    .await
                    .map_err(RegistryError::NotificationError);
                let view = notification.and_then(|n| {
                    serde_json::from_str::<View>(n.payload())
                        .map_err(RegistryError::MalformedNotification)
                });

                if tx.send(view).is_err() {
                    return;
                }
            }
        });

        Ok(rx)
    }

    pub async fn import_all(&self, imported: DbExport) -> RegistryResult<()> {
        let mut conn = self.connect().await?;
        if !self.get_all_schemas().await?.is_empty() {
            warn!("[IMPORT] Database is not empty, skipping importing");
            return Ok(());
        }

        conn
            .transaction::<_, _, RegistryError>(move |c| {
                Box::pin(async move {
                    for schema in imported.schemas {
                        sqlx::query!(
                            "INSERT INTO schemas(id, name, schema_type, insert_destination, query_address) \
                             VALUES($1, $2, $3, $4, $5)",
                            schema.id,
                            schema.name,
                            schema.schema_type as _,
                            schema.insert_destination,
                            schema.query_address
                        )
                        .execute(c.acquire().await?)
                        .await?;

                        for definition in schema.definitions {
                            sqlx::query!(
                                "INSERT INTO definitions(version, definition, schema) \
                                 VALUES($1, $2, $3)",
                                definition.version.to_string(),
                                definition.definition,
                                schema.id
                            )
                            .execute(c.acquire().await?)
                            .await?;
                        }

                        for view in schema.views {
                            sqlx::query!(
                                "INSERT INTO views(id, schema, name, materializer_address, materializer_options, fields) \
                                 VALUES($1, $2, $3, $4, $5, $6)",
                                 view.id,
                                 schema.id,
                                 view.name,
                                 view.materializer_address,
                                 view.materializer_options,
                                 serde_json::to_value(&view.fields)
                                    .map_err(RegistryError::MalformedViewFields)?,
                            )
                            .execute(c.acquire().await?)
                            .await?;
                        }
                     }

                    Ok(())
                })
            })
            .await?;

        Ok(())
    }

    pub async fn export_all(&self) -> RegistryResult<DbExport> {
        Ok(DbExport {
            schemas: self.get_all_full_schemas().await?,
        })
    }
}
