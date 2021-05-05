use opentelemetry::global;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use tokio::runtime::Handle;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

#[cfg(feature = "grpc")]
pub mod grpc;
#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "kafka")]
pub mod kafka;

pub fn init<'a>(rust_log: impl Into<Option<&'a str>>) -> anyhow::Result<()> {
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

    let filter = rust_log.into().map(EnvFilter::new).unwrap_or_else(|| EnvFilter::from_default_env());

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt)
        .with(opentelemetry)
        .try_init()?;

    Ok(())
}
