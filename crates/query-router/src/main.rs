use cache::SchemaRegistryCache;
use std::sync::Arc;
use structopt::StructOpt;
use utils::metrics;
use uuid::Uuid;
use warp::Filter;

pub mod cache;
pub mod error;
pub mod handler;

#[derive(StructOpt)]
struct Config {
    /// Address of schema registry gRPC API
    #[structopt(long, env = "SCHEMA_REGISTRY_ADDR")]
    schema_registry_addr: String,
    /// How many entries the cache can hold
    #[structopt(long, env = "CACHE_CAPACITY")]
    cache_capacity: usize,
    /// Port to listen on
    #[structopt(long, env = "INPUT_PORT")]
    input_port: u16,
    /// Port to listen on for Prometheus requests
    #[structopt(default_value = metrics::DEFAULT_PORT, env)]
    pub metrics_port: u16,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = Config::from_args();

    metrics::serve(config.metrics_port);

    let schema_registry_cache = Arc::new(SchemaRegistryCache::new(
        config.schema_registry_addr,
        config.cache_capacity,
    ));

    let address_filter = warp::any().map(move || schema_registry_cache.clone());
    let schema_id_filter = warp::header::header::<Uuid>("SCHEMA_ID");
    let body_filter = warp::body::content_length_limit(1024 * 32).and(warp::body::json());

    let single_route = warp::path!("single" / Uuid)
        .and(schema_id_filter)
        .and(address_filter.clone())
        .and(body_filter)
        .and_then(handler::query_single);
    let multiple_route = warp::path!("multiple" / String)
        .and(schema_id_filter)
        .and(address_filter.clone())
        .and_then(handler::query_multiple);
    let schema_route = warp::path!("schema")
        .and(schema_id_filter)
        .and(address_filter.clone())
        .and_then(handler::query_by_schema);
    let raw_route = warp::path!("raw")
        .and(schema_id_filter)
        .and(address_filter.clone())
        .and(body_filter)
        .and_then(handler::query_raw);

    let routes = warp::post()
        .and(single_route.or(raw_route))
        .or(warp::get().and(multiple_route.or(schema_route)));

    warp::serve(routes)
        .run(([0, 0, 0, 0], config.input_port))
        .await;
}
