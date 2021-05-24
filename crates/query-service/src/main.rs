use anyhow::Context;
use metrics_utils as metrics;
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
}

async fn spawn_server<Q: QueryService>(service: Q, port: u16) -> anyhow::Result<()> {
    let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port);

    Server::builder()
        .trace_fn(tracing_utils::grpc::trace_fn)
        .add_service(QueryServiceServer::new(service))
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

    spawn_server(
        query_service::psql::PsqlQuery::load(settings.postgres).await?,
        settings.input_port,
    )
    .await
}
