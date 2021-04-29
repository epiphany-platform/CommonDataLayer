use anyhow::Context;
use clap::Clap;
use query_service_ts::druid::{DruidConfig, DruidQuery};
use query_service_ts::victoria::{VictoriaConfig, VictoriaQuery};
use rpc::query_service_ts::query_service_ts_server::{QueryServiceTs, QueryServiceTsServer};
use std::net::{Ipv4Addr, SocketAddrV4};
use tonic::transport::Server;
use utils::metrics;

#[derive(Clap)]
pub struct Config {
    #[clap(subcommand)]
    pub inner: ConfigType,
    #[clap(long, env = "INPUT_PORT")]
    pub input_port: u16,
    #[clap(default_value = metrics::DEFAULT_PORT, env)]
    pub metrics_port: u16,
}

#[derive(Clap)]
pub enum ConfigType {
    Victoria(VictoriaConfig),
    Druid(DruidConfig),
}

//Could be extracted to utils, dunno how without schema
async fn spawn_server<Q: QueryServiceTs>(service: Q, port: u16) -> anyhow::Result<()> {
    let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port);

    Server::builder()
        .trace_fn(utils::tracing::grpc::trace_fn)
        .add_service(QueryServiceTsServer::new(service))
        .serve(addr.into())
        .await
        .context("gRPC server failed")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();

    let config: Config = Config::parse();
    utils::tracing::init();
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
