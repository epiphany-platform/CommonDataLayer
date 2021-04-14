use opentelemetry::global;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use tracing_subscriber::prelude::*;

pub mod grpc;
pub mod kafka; // Used mostly in common publisher and [parallel_]consumer

pub fn init() {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let tracer = opentelemetry_jaeger::new_pipeline()
        .install_batch(opentelemetry::runtime::Tokio)
        .unwrap();

    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let fmt = tracing_subscriber::fmt::layer();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(fmt)
        .with(opentelemetry)
        .try_init()
        .unwrap();
}
