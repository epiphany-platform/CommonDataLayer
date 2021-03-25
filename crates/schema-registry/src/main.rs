use std::fs::File;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::PathBuf;

use anyhow::Context;
use structopt::{clap::arg_enum, StructOpt};
use tonic::transport::Server;
use utils::{metrics, status_endpoints};

use rpc::schema_registry::schema_registry_server::SchemaRegistryServer;
use schema_registry::config::{communication_config, Config};
use schema_registry::rpc::SchemaRegistryImpl;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();
    utils::tracing::init();
    let config = Config::from_args();

    status_endpoints::serve();
    metrics::serve(
        config
            .metrics_port
            .unwrap_or_else(|| metrics::DEFAULT_PORT.parse().unwrap()),
    );

    let comms_config = communication_config(&config)?;
    let registry = SchemaRegistryImpl::new(config.db_name, comms_config).await?;

    if let Some(export_filename) = config.export_dir.map(export_path) {
        let exported = registry.export_all().await?;
        let file = File::create(export_filename)?;
        serde_json::to_writer_pretty(&file, &exported)?;
    }

    if let Some(import_path) = config.import_file {
        let imported = File::open(import_path)?;
        let imported = serde_json::from_reader(imported)?;
        registry.import_all(imported).await?;
    }

    let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), config.input_port);

    Server::builder()
        .add_service(SchemaRegistryServer::new(registry))
        .serve(addr.into())
        .await
        .context("gRPC server failed")
}

fn export_path(export_dir_path: PathBuf) -> PathBuf {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Invalid system time");

    export_dir_path.join(format!("export_{:?}.json", timestamp.as_secs()))
}
