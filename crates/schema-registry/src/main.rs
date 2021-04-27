use std::net::{Ipv4Addr, SocketAddrV4};

use anyhow::Context;
use clap::Clap;
use tokio::time::sleep;
use tokio::time::Duration;
use tonic::transport::Server;

use rpc::schema_registry::schema_registry_server::SchemaRegistryServer;
use schema_registry::config::{communication_config, Config};
use schema_registry::rpc::SchemaRegistryImpl;
use utils::{metrics, status_endpoints};

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();
    utils::tracing::init();
    let config = Config::parse();

    sleep(Duration::from_millis(500)).await;

    status_endpoints::serve(config.status_port);
    metrics::serve(config.metrics_port);

    let comms_config = communication_config(&config)?;
    let registry = SchemaRegistryImpl::new(&config, comms_config).await?;

    let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), config.input_port);
    status_endpoints::mark_as_started();
    Server::builder()
        .add_service(SchemaRegistryServer::new(registry))
        .serve(addr.into())
        .await
        .context("gRPC server failed")
}
