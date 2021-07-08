use anyhow::Context;
use metrics_utils as metrics;
use query_service::psql::PsqlQuery;
use rpc::query_service::query_service_raw_server::{QueryServiceRaw, QueryServiceRawServer};
use rpc::query_service::query_service_server::{QueryService, QueryServiceServer};
use serde::Deserialize;
use settings_utils::*;
use std::net::{Ipv4Addr, SocketAddrV4};
use tonic::transport::Server;

#[derive(Debug, Deserialize)]
pub struct Settings {
    postgres: PostgresSettings,
    input_port: u16,

    monitoring: MonitoringSettings,

    #[serde(default)]
    log: LogSettings,

    #[serde(default)]
    features: FeatureSettings,
}

#[derive(Debug, Deserialize)]
pub struct FeatureSettings {
    pub raw_endpoint: bool,
}

impl Default for FeatureSettings {
    fn default() -> Self {
        Self { raw_endpoint: true }
    }
}

async fn spawn_server<Q: QueryService, R: QueryServiceRaw>(
    raw_service: Option<R>,
    main_service: Q,
    port: u16,
) -> anyhow::Result<()> {
    let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port);

    Server::builder()
        .trace_fn(tracing_utils::grpc::trace_fn)
        .add_service(QueryServiceServer::new(main_service))
        .add_optional_service(raw_service.map(QueryServiceRawServer::new))
        .serve(addr.into())
        .await
        .context("gRPC server failed")
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

    let postgres_query = PsqlQuery::load(settings.postgres).await?;
    spawn_server(
        settings.features.raw_endpoint.then(|| postgres_query.clone()),
        postgres_query,
        settings.input_port,
    )
    .await
}
