use std::convert::TryInto;
use std::sync::{Arc, Mutex};

use lru_cache::LruCache;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::error::{CacheError, CacheResult};
use crate::types::schema::Schema;

#[derive(Clone)]
pub struct SchemaCache {
    schemas: Arc<Mutex<LruCache<Uuid, Arc<Schema>>>>,
    schema_registry_addr: String,
}

impl SchemaCache {
    pub async fn new(
        schema_registry_addr: String,
        capacity: usize,
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

        let schemas = Arc::new(Mutex::new(LruCache::new(capacity)));
        let schemas2 = Arc::clone(&schemas);

        tokio::spawn(async move {
            loop {
                let message = schema_updates
                    .message()
                    .await
                    .map_err(CacheError::SchemaUpdateReceiveError);
                match message {
                    Ok(Some(schema)) => match Self::parse_schema(schema) {
                        Ok(schema) => {
                            if let Some(entry) = schemas2.lock().unwrap().get_mut(&schema.id) {
                                *entry = Arc::new(schema);
                            }
                        }
                        Err(error) => {
                            tx.send(error).ok();
                            break;
                        }
                    },
                    Ok(None) => {
                        break;
                    }
                    Err(error) => {
                        tx.send(error).ok();
                        break;
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
            insert_destination: schema.metadata.insert_destination,
            schema_type: schema
                .metadata
                .schema_type
                .try_into()
                .map_err(CacheError::RegistryError)?,
        })
    }

    async fn retrieve_schema(id: Uuid, schema_registry_addr: String) -> CacheResult<Schema> {
        let mut conn = rpc::schema_registry::connect(schema_registry_addr)
            .await
            .map_err(CacheError::ConnectionError)?;
        let metadata = conn
            .get_schema_metadata(rpc::schema_registry::Id { id: id.to_string() })
            .await
            .map_err(CacheError::RegistryError)?
            .into_inner();

        Ok(Schema {
            id,
            name: metadata.name,
            query_address: metadata.query_address,
            insert_destination: metadata.insert_destination,
            schema_type: metadata
                .schema_type
                .try_into()
                .map_err(CacheError::RegistryError)?,
        })
    }

    pub async fn get_schema(&self, id: Uuid) -> CacheResult<Arc<Schema>> {
        if !self.schemas.lock().unwrap().contains_key(&id) {
            let schema = Self::retrieve_schema(id, self.schema_registry_addr.clone()).await?;
            self.schemas.lock().unwrap().insert(id, Arc::new(schema));
        }

        Ok(self
            .schemas
            .lock()
            .unwrap()
            .get_mut(&id)
            .ok_or(CacheError::MissingSchema)?
            .clone())
    }
}
