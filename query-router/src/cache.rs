use crate::error::Error;
use lru_cache::LruCache;
use rpc::error::ClientError;
use rpc::schema_registry::types::SchemaType;
use std::sync::{Mutex, MutexGuard};
use utils::abort_on_poison;
use uuid::Uuid;

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
        let response = conn
            .get_schema_query_address(rpc::schema_registry::Id {
                id: schema_id.to_string(),
            })
            .await
            .map_err(|err| {
                Error::ClientError(ClientError::QueryError {
                    service: "schema registry",
                    source: err,
                })
            })?;
        let address = response.into_inner().address;

        let response = conn
            .get_schema_type(rpc::schema_registry::Id {
                id: schema_id.to_string(),
            })
            .await
            .map_err(|err| {
                Error::ClientError(ClientError::QueryError {
                    service: "schema registry",
                    source: err,
                })
            })?;
        let schema_type = response.into_inner().schema_type().into();

        let result = (address, schema_type);
        self.lock().insert(schema_id, result.clone());

        Ok(result)
    }
}
