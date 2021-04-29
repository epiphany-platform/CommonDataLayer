use std::fs::File;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::PathBuf;

use anyhow::Context;
use tokio::time::sleep;
use tokio::time::Duration;
use tonic::transport::Server;

use rpc::schema_registry::schema_registry_server::SchemaRegistryServer;
use schema_registry::config::Settings;
use schema_registry::rpc::SchemaRegistryImpl;
use utils::settings::load_settings;
use utils::{metrics, status_endpoints};

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();
    utils::tracing::init();
    let settings: Settings = load_settings()?;

    sleep(Duration::from_millis(500)).await;

    status_endpoints::serve(&settings.monitoring);
    metrics::serve(&settings.monitoring);

    let registry = SchemaRegistryImpl::new(&settings).await?;

    if let Some(export_filename) = settings.export_dir.map(export_path) {
        let exported = registry.export_all().await?;
        let file = File::create(export_filename)?;
        serde_json::to_writer_pretty(&file, &exported)?;
    }

    if let Some(import_path) = settings.import_file {
        let imported = File::open(import_path).map_err(|err| anyhow::anyhow!("{}", err))?;
        let imported = serde_json::from_reader(imported)?;
        registry
            .import_all(imported)
            .await
            .map_err(|err| anyhow::anyhow!("Failed to import database: {}", err))?;
    }

    let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), settings.input_port);
    status_endpoints::mark_as_started();
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
