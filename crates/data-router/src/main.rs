use crate::schema::SchemaMetadataFetcher;
use crate::{config::Settings, handler::Handler};
use cache::DynamicCache;
use metrics_utils as metrics;
use settings_utils::load_settings;
use std::sync::Arc;
use tokio::sync::Mutex;
use utils::parallel_task_queue::ParallelTaskQueue;

mod config;
mod handler;
mod schema;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    misc_utils::set_aborting_panic_hook();

    let settings: Settings = load_settings()?;
    tracing_utils::init(
        settings.log.rust_log.as_str(),
        settings.monitoring.otel_service_name.as_str(),
    )?;

    tracing::debug!(?settings, "application environment");

    metrics::serve(&settings.monitoring);

    let consumer = settings.consumer().await?;
    let producer = Arc::new(settings.producer().await?);

    let cache = Arc::new(Mutex::new(DynamicCache::new(
        settings.cache_capacity,
        SchemaMetadataFetcher::boxed(settings.services.schema_registry_url),
    )));

    let task_queue = Arc::new(ParallelTaskQueue::default());

    consumer
        .par_run(Handler {
            cache,
            producer,
            task_queue,
            routing_table: Arc::new(settings.repositories),
        })
        .await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    Ok(())
}
