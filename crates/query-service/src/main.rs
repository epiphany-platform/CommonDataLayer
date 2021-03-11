use anyhow::Context;
use rpc::query_service::query_service_server::{QueryService, QueryServiceServer};
use std::net::{Ipv4Addr, SocketAddrV4};
use structopt::StructOpt;
use tonic::transport::Server;
use utils::metrics;

#[derive(StructOpt)]
pub struct Config {
    #[structopt(subcommand)]
    pub inner: ConfigType,
    /// Port to listen on
    #[structopt(long, env)]
    pub input_port: u16,
    /// Port to listen on for Prometheus requests
    #[structopt(default_value = metrics::DEFAULT_PORT, env)]
    pub metrics_port: u16,
}

#[derive(StructOpt)]
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
    env_logger::init();

    let config: Config = Config::from_args();

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
