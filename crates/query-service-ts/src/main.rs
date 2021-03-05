use anyhow::Context;
use query_service_ts::druid::{DruidConfig, DruidQuery};
use query_service_ts::victoria::{VictoriaConfig, VictoriaQuery};
use rpc::query_service_ts::query_service_ts_server::{QueryServiceTs, QueryServiceTsServer};
use std::net::{Ipv4Addr, SocketAddrV4};
use structopt::StructOpt;
use tonic::transport::Server;
use utils::metrics;

#[derive(StructOpt)]
pub struct Config {
    #[structopt(subcommand)]
    pub inner: ConfigType,
    #[structopt(long, env = "INPUT_PORT")]
    pub input_port: u16,
    #[structopt(default_value = metrics::DEFAULT_PORT, env)]
    pub metrics_port: u16,
}

#[derive(StructOpt)]
pub enum ConfigType {
    Victoria(VictoriaConfig),
    Druid(DruidConfig),
}

//Could be extracted to utils, dunno how without schema
async fn spawn_server<Q: QueryServiceTs>(service: Q, port: u16) -> anyhow::Result<()> {
    let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port);

    Server::builder()
        .add_service(QueryServiceTsServer::new(service))
        .serve(addr.into())
        .await
        .context("gRPC server failed")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config: Config = Config::from_args();
    env_logger::init();
    metrics::serve(config.metrics_port);

    match config.inner {
        ConfigType::Victoria(victoria_config) => {
            spawn_server(
                VictoriaQuery::load(victoria_config).await?,
                config.input_port,
            )
            .await
        }
        ConfigType::Druid(druid_config) => {
            spawn_server(DruidQuery::load(druid_config).await?, config.input_port).await
        }
    }
}
