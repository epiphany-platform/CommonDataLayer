use anyhow::Context;
use chrono::{DateTime, Utc};
use indradb::SledDatastore;
use rpc::schema_registry::schema_registry_server::SchemaRegistryServer;
use schema_registry::{
    error::RegistryError,
    replication::{KafkaConfig, ReplicationRole},
    rpc::SchemaRegistryImpl,
};
use serde::Deserialize;
use std::fs::File;
use std::io::Write;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::PathBuf;
use tonic::transport::Server;
use utils::{metrics, status_endpoints};

#[derive(Deserialize)]
struct Config {
    pub input_port: u16,
    pub db_name: String,
    pub replication_role: ReplicationRole,
    pub kafka_brokers: String,
    pub kafka_group_id: String,
    pub kafka_topics: Vec<String>,
    pub pod_name: Option<String>,
    pub export_dir: Option<PathBuf>,
    pub import_file: Option<PathBuf>,
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let config = envy::from_env::<Config>().context("Env vars not set correctly")?;
    let replication_config = KafkaConfig {
        brokers: config.kafka_brokers,
        group_id: config.kafka_group_id,
        topics: config.kafka_topics,
    };

    status_endpoints::serve();
    metrics::serve();

    let data_store = SledDatastore::new(&config.db_name).map_err(RegistryError::ConnectionError)?;
    let registry = SchemaRegistryImpl::new(
        data_store,
        config.replication_role,
        replication_config,
        config.pod_name,
    )
    .await?;

    if let Some(export_dir_path) = config.export_dir {
        let exported = registry.export_all()?;
        let exported = serde_json::to_string(&exported)?;
        let export_path = export_path(export_dir_path);
        let mut file = File::create(export_path)?;
        write!(file, "{}", exported)?;
    }

    if let Some(import_path) = config.import_file {
        let imported = File::open(import_path)?;
        let imported = serde_json::from_reader(imported)?;
        registry.import_all(imported)?;
    }

    let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), config.input_port);
    status_endpoints::mark_as_started();
    Server::builder()
        .add_service(SchemaRegistryServer::new(registry))
        .serve(addr.into())
        .await?;

    Ok(())
}

fn export_path(export_dir_path: PathBuf) -> PathBuf {
    let timestamp: DateTime<Utc> = Utc::now();
    export_dir_path.join(format!("export_{:?}.json", timestamp))
}
