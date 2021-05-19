use std::sync::Arc;

use uuid::Uuid;
use warp::Filter;

use cache::SchemaRegistryCache;
use serde::Deserialize;
use utils::metrics;
use utils::settings::{load_settings, LogSettings, MonitoringSettings};

pub mod cache;
pub mod error;
pub mod handler;

#[derive(Debug, Deserialize)]
struct Settings {
    cache_capacity: usize,
    input_port: u16,

    services: ServicesSettings,

    monitoring: MonitoringSettings,

    #[serde(default)]
    log: LogSettings,
}

#[derive(Debug, Deserialize)]
struct ServicesSettings {
    schema_registry_url: String,
}

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

    let schema_registry_cache = Arc::new(SchemaRegistryCache::new(
        settings.services.schema_registry_url,
        settings.cache_capacity,
    ));

    let cache_filter = warp::any().map(move || schema_registry_cache.clone());
    let schema_id_filter = warp::header::header::<Uuid>("SCHEMA_ID");
    let body_filter = warp::body::content_length_limit(1024 * 32).and(warp::body::json());

    let single_route = warp::path!("single" / Uuid)
        .and(schema_id_filter)
        .and(cache_filter.clone())
        .and(body_filter)
        .and_then(handler::query_single);

    let multiple_route = warp::path!("multiple" / String)
        .and(schema_id_filter)
        .and(cache_filter.clone())
        .and_then(handler::query_multiple);

    let schema_route = warp::path!("schema")
        .and(schema_id_filter)
        .and(cache_filter.clone())
        .and_then(handler::query_by_schema);

    let raw_route = warp::path!("raw")
        .and(schema_id_filter)
        .and(cache_filter.clone())
        .and(body_filter)
        .and_then(handler::query_raw);

    let routes = warp::post()
        .and(single_route.or(raw_route))
        .or(warp::get().and(multiple_route.or(schema_route)));

    tracing_utils::http::serve(routes, ([0, 0, 0, 0], settings.input_port)).await;

    Ok(())
}
