use std::sync::Mutex;

use lru_cache::LruCache;
use tracing::trace;

use misc_utils::abort_on_poison;
use rpc::schema_registry::Id;
use uuid::Uuid;

#[tracing::instrument(skip(cache))]
pub async fn get_schema_insert_destination(
    cache: &Mutex<LruCache<Uuid, String>>,
    schema_id: Uuid,
    schema_addr: &str,
) -> anyhow::Result<String> {
    let destination = cache
        .lock()
        .unwrap_or_else(abort_on_poison)
        .get_mut(&schema_id)
        .cloned();
    if let Some(dest) = destination {
        trace!("Retrieved insert destination for {} from cache", schema_id);
        return Ok(dest);
    }

    let mut client = rpc::schema_registry::connect(schema_addr.to_owned()).await?;
    let insert_destination = client
        .get_schema_metadata(Id {
            id: schema_id.to_string(),
        })
        .await?
        .into_inner()
        .insert_destination;

    trace!(
        "Retrieved insert destination for {} from schema registry",
        schema_id
    );
    cache
        .lock()
        .unwrap_or_else(abort_on_poison)
        .insert(schema_id, insert_destination.clone());

    Ok(insert_destination)
}
