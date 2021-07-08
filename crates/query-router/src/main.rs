use crate::schema::SchemaMetadata;
use cache::DynamicCache;
use metrics_utils as metrics;
use schema::SchemaMetadataSupplier;
use serde::Deserialize;
use settings_utils::{load_settings, LogSettings, MonitoringSettings, RepositoryStaticRouting};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use warp::Filter;

pub mod error;
pub mod handler;
pub mod schema;

#[derive(Debug, Deserialize)]
struct Settings {
    cache_capacity: usize,
    input_port: u16,

    services: ServicesSettings,

    monitoring: MonitoringSettings,

    #[serde(default)]
    log: LogSettings,

    #[serde(default)]
    repositories: HashMap<String, RepositoryStaticRouting>,

    #[serde(default)]
    features: FeatureSettings,
}

#[derive(Debug, Deserialize)]
struct ServicesSettings {
    schema_registry_url: String,
}

#[derive(Debug, Deserialize)]
struct FeatureSettings {
    raw_endpoint: bool,
}

impl Default for FeatureSettings {
    fn default() -> Self {
        Self { raw_endpoint: true }
    }
}

pub struct Config {
    cache: DynamicCache<SchemaMetadataSupplier, Uuid, SchemaMetadata>,
    routing: HashMap<String, RepositoryStaticRouting>,
}

#[derive(Debug)]
pub struct Headers {
    schema_id: Uuid,
    repository_id: Option<String>,
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

    let config = Arc::new(Config {
        cache: DynamicCache::new(
            settings.cache_capacity,
            SchemaMetadataSupplier::new(settings.services.schema_registry_url),
        ),
        routing: settings.repositories,
    });

    let config_filter = warp::any().map(move || config.clone());

    let common_headers = warp::header::header::<Uuid>("SCHEMA_ID")
        .and(warp::header::optional::<String>("REPOSITORY_ID"))
        .map(|schema_id, repository_id| Headers {
            schema_id,
            repository_id,
        });

    let body_filter = warp::body::content_length_limit(1024 * 32).and(warp::body::json());

    let single_route = warp::path!("single" / Uuid)
        .and(common_headers)
        .and(config_filter.clone())
        .and(body_filter)
        .and_then(handler::query_single);

    let multiple_route = warp::path!("multiple" / String)
        .and(common_headers)
        .and(config_filter.clone())
        .and_then(handler::query_multiple);

    let schema_route = warp::path!("schema")
        .and(common_headers)
        .and(config_filter.clone())
        .and_then(handler::query_by_schema);

    if settings.features.raw_endpoint {
        let raw_route = warp::path!("raw")
            .and(common_headers)
            .and(config_filter)
            .and(body_filter)
            .and_then(handler::query_raw);

        let post_routes = warp::post().and(single_route.or(raw_route));

        let get_routes = warp::get().and(multiple_route.or(schema_route));

        let routes = post_routes.or(get_routes);

        tracing_utils::http::serve(routes, ([0, 0, 0, 0], settings.input_port)).await;
    } else {
        let get_routes = warp::get().and(multiple_route.or(schema_route));

        let routes = get_routes.or(single_route);

        tracing_utils::http::serve(routes, ([0, 0, 0, 0], settings.input_port)).await;
    }

    Ok(())
}
