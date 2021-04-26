use std::convert::TryInto;
use std::sync::{Mutex, MutexGuard};

use lru_cache::LruCache;
use uuid::Uuid;

use crate::error::Error;
use rpc::error::ClientError;
use rpc::schema_registry::types::SchemaType;
use utils::abort_on_poison;

pub struct SchemaRegistryCache {
    schema_registry_addr: String,
    cache: Mutex<LruCache<Uuid, (String, SchemaType)>>,
}

impl SchemaRegistryCache {
    pub fn new(schema_registry_addr: String, capacity: usize) -> Self {
        Self {
            schema_registry_addr,
            cache: Mutex::new(LruCache::new(capacity)),
        }
    }

    fn lock(&self) -> MutexGuard<LruCache<Uuid, (String, SchemaType)>> {
        self.cache.lock().unwrap_or_else(abort_on_poison)
    }

    pub async fn get_schema_info(&self, schema_id: Uuid) -> Result<(String, SchemaType), Error> {
        if let Some(cached_data) = self.lock().get_mut(&schema_id) {
            return Ok(cached_data.clone());
        }

        let mut conn = rpc::schema_registry::connect(self.schema_registry_addr.clone())
            .await
            .map_err(Error::ClientError)?;
        let metadata = conn
            .get_schema_metadata(rpc::schema_registry::Id {
                id: schema_id.to_string(),
            })
            .await
            .map_err(|err| {
                Error::ClientError(ClientError::QueryError {
                    service: "schema registry",
                    source: err,
                })
            })?
            .into_inner();

        let result = (
            metadata.query_address,
            metadata
                .schema_type
                .try_into()
                .map_err(Error::InvalidSchemaType)?,
        );
        self.lock().insert(schema_id, result.clone());

        Ok(result)
    }
}
