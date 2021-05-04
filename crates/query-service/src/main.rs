use anyhow::Context;
use rpc::query_service::query_service_server::{QueryService, QueryServiceServer};
use serde::Deserialize;
use std::net::{Ipv4Addr, SocketAddrV4};
use tonic::transport::Server;
use utils::metrics;
use utils::settings::*;

#[derive(Debug, Deserialize)]
pub struct Settings {
    postgres: PostgresSettings,
    input_port: u16,

    monitoring: MonitoringSettings,

    log: LogSettings,
}

async fn spawn_server<Q: QueryService>(service: Q, port: u16) -> anyhow::Result<()> {
    let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port);

    Server::builder()
        .add_service(QueryServiceServer::new(service))
        .serve(addr.into())
        .await
        .context("gRPC server failed")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();

    let settings: Settings = load_settings()?;
    settings.log.init()?;

    tracing::debug!(?settings, "command-line arguments");

    metrics::serve(&settings.monitoring);

    spawn_server(
        query_service::psql::PsqlQuery::load(settings.postgres).await?,
        settings.input_port,
    )
    .await
}
