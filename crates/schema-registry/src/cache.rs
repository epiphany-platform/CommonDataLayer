use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::{Arc, RwLock};
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::error::{CacheError, CacheResult};
use crate::types::Schema;

#[derive(Clone)]
pub struct SchemaCache {
    schemas: Arc<RwLock<HashMap<Uuid, Arc<Schema>>>>,
    schema_registry_addr: String,
}

impl SchemaCache {
    pub async fn new(
        schema_registry_addr: String,
    ) -> CacheResult<(Self, oneshot::Receiver<CacheError>)> {
        let (tx, rx) = oneshot::channel::<CacheError>();
        let mut conn = rpc::schema_registry::connect(schema_registry_addr.clone())
            .await
            .map_err(CacheError::ConnectionError)?;
        let mut schema_updates = conn
            .watch_all_schema_updates(rpc::schema_registry::Empty {})
            .await
            .map_err(CacheError::RegistryError)?
            .into_inner();

        let schemas = Arc::new(RwLock::new(HashMap::new()));
        let schemas2 = Arc::clone(&schemas);

        tokio::spawn(async move {
            'receive: loop {
                let message = schema_updates
                    .message()
                    .await
                    .map_err(CacheError::SchemaUpdateReceiveError);
                match message {
                    Ok(Some(schema)) => match Self::parse_schema(schema) {
                        Ok(schema) => {
                            schemas2
                                .write()
                                .unwrap()
                                .entry(schema.id)
                                .and_modify(|s| *s = Arc::new(schema));
                        }
                        Err(error) => {
                            tx.send(error).ok();
                            break 'receive;
                        }
                    },
                    Ok(None) => {
                        break 'receive;
                    }
                    Err(error) => {
                        tx.send(error).ok();
                        break 'receive;
                    }
                }
            }
        });

        Ok((
            SchemaCache {
                schemas,
                schema_registry_addr,
            },
            rx,
        ))
    }

    pub fn parse_schema(schema: rpc::schema_registry::Schema) -> CacheResult<Schema> {
        Ok(Schema {
            id: Uuid::parse_str(&schema.id).map_err(|_err| CacheError::MalformedSchema)?,
            name: schema.metadata.name,
            query_address: schema.metadata.query_address,
            topic_or_queue: schema.metadata.topic_or_queue,
            r#type: schema
                .metadata
                .r#type
                .try_into()
                .map_err(CacheError::RegistryError)?,
        })
    }

    async fn retrieve_schema(id: Uuid, schema_registry_addr: String) -> CacheResult<Schema> {
        let mut conn = rpc::schema_registry::connect(schema_registry_addr)
            .await
            .map_err(CacheError::ConnectionError)?;
        let metadata = conn
            .get_schema(rpc::schema_registry::Id { id: id.to_string() })
            .await
            .map_err(CacheError::RegistryError)?
            .into_inner()
            .metadata;

        Ok(Schema {
            id,
            name: metadata.name,
            query_address: metadata.query_address,
            topic_or_queue: metadata.topic_or_queue,
            r#type: metadata
                .r#type
                .try_into()
                .map_err(CacheError::RegistryError)?,
        })
    }

    pub async fn get_schema(&self, id: Uuid) -> CacheResult<Arc<Schema>> {
        if !self.schemas.read().unwrap().contains_key(&id) {
            let schema = Self::retrieve_schema(id, self.schema_registry_addr.clone()).await?;
            self.schemas.write().unwrap().insert(id, Arc::new(schema));
        }

        Ok(self
            .schemas
            .read()
            .unwrap()
            .get(&id)
            .ok_or(CacheError::MissingSchema)?
            .clone())
    }
}
