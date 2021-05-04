use opentelemetry::global;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use tokio::runtime::Handle;
use tracing_subscriber::prelude::*;

pub mod grpc;
pub mod http;
pub mod kafka;

pub fn init() -> anyhow::Result<()> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let opentelemetry = Handle::try_current()
        .ok() // Check if Tokio runtime exists
        .and_then(|_| {
            opentelemetry_jaeger::new_pipeline()
                .install_batch(opentelemetry::runtime::Tokio)
                .ok()
        })
        .map(|tracer| tracing_opentelemetry::layer().with_tracer(tracer));

    let fmt = tracing_subscriber::fmt::layer();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(fmt)
        .with(opentelemetry)
        .try_init()?;

    Ok(())
}
