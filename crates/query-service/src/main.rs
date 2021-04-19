use anyhow::Context;
use clap::Clap;
use rpc::query_service::query_service_server::{QueryService, QueryServiceServer};
use std::net::{Ipv4Addr, SocketAddrV4};
use tonic::transport::Server;
use utils::metrics;

#[derive(Clap, Debug)]
pub struct Config {
    #[clap(subcommand)]
    pub inner: ConfigType,
    /// Port to listen on
    #[clap(long, env)]
    pub input_port: u16,
    /// Port to listen on for Prometheus requests
    #[clap(default_value = metrics::DEFAULT_PORT, env)]
    pub metrics_port: u16,
}

#[derive(Clap, Debug)]
pub enum ConfigType {
    Postgres(query_service::psql::PsqlConfig),
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
    utils::tracing::init();

    let config: Config = Config::parse();

    tracing::debug!(?config, "Config");

    metrics::serve(config.metrics_port);

    match config.inner {
        ConfigType::Postgres(psql_config) => {
            spawn_server(
                query_service::psql::PsqlQuery::load(psql_config).await?,
                config.input_port,
            )
            .await
        }
    }
}
