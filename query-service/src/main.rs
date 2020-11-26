use anyhow::Context;
use query_service::schema::query_server::{Query, QueryServer};
use std::net::{Ipv4Addr, SocketAddrV4};
use structopt::StructOpt;
use tonic::transport::Server;
use utils::metrics;

#[derive(StructOpt)]
pub struct Config {
    #[structopt(subcommand)]
    pub inner: ConfigType,
    #[structopt(long, env)]
    pub input_port: u16,
}

#[derive(StructOpt)]
pub enum ConfigType {
    Postgres(query_service::psql::PsqlConfig),
    Sled(query_service::ds::DsConfig),
}

async fn spawn_server<Q: Query>(service: Q, port: u16) -> anyhow::Result<()> {
    let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port);

    Server::builder()
        .add_service(QueryServer::new(service))
        .serve(addr.into())
        .await
        .context("gRPC server failed")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config: Config = Config::from_args();

    metrics::serve();

    match config.inner {
        ConfigType::Postgres(psql_config) => {
            spawn_server(
                query_service::psql::PsqlQuery::load(psql_config).await?,
                config.input_port,
            )
            .await
        }
        ConfigType::Sled(sled_config) => {
            spawn_server(
                query_service::ds::DsQuery::load(sled_config).await?,
                config.input_port,
            )
            .await
        }
    }
}
