use crate::error::Error;
use lru_cache::LruCache;
use rpc::error::ClientError;
use std::sync::{Mutex, MutexGuard};
use utils::abort_on_poison;
use uuid::Uuid;

pub struct AddressCache {
    schema_registry_addr: String,
    addresses: Mutex<LruCache<Uuid, String>>,
}

impl AddressCache {
    pub fn new(schema_registry_addr: String, capacity: usize) -> Self {
        Self {
            schema_registry_addr,
            addresses: Mutex::new(LruCache::new(capacity)),
        }
    }

    fn lock(&self) -> MutexGuard<LruCache<Uuid, String>> {
        self.addresses.lock().unwrap_or_else(abort_on_poison)
    }

    pub async fn get_address(&self, schema_id: Uuid) -> Result<String, Error> {
        if let Some(address) = self.lock().get_mut(&schema_id) {
            return Ok(address.clone());
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

        self.lock().insert(schema_id, address.clone());

        Ok(address)
    }
}
