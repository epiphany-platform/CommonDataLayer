use anyhow::Context;
use chrono::{DateTime, Utc};
use indradb::SledDatastore;
use rpc::schema_registry::schema_registry_server::SchemaRegistryServer;
use schema_registry::{
    error::RegistryError,
    replication::CommunicationMethod,
    replication::{ReplicationMethodConfig, ReplicationRole},
    rpc::SchemaRegistryImpl,
    AmqpConfig, CommunicationMethodConfig, KafkaConfig,
};
use serde::Deserialize;
use std::fs::File;
use std::io::Write;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::PathBuf;
use tonic::transport::Server;
use utils::{metrics, status_endpoints};

enum CommunicationMethodType {
    Kafka,
    Amqp,
    Grpc,
}

impl<'de> Deserialize<'de> for CommunicationMethodType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?.to_lowercase();
        let mqt = match s.as_str() {
            "kafka" => Self::Kafka,
            "amqp" => Self::Amqp,
            "grpc" => Self::Grpc,
            other => {
                return Err(serde::de::Error::custom(format!(
                    "Invalid message queue type: `{}`",
                    other
                )));
            }
        };
        Ok(mqt)
    }
}

#[derive(Deserialize)]
struct Config {
    pub input_port: u16,
    pub db_name: String,
    pub replication_role: ReplicationRole,

    pub communication_method: CommunicationMethodType,
    pub kafka_brokers: Option<String>,
    pub kafka_group_id: Option<String>,
    pub amqp_connection_string: Option<String>,
    pub amqp_consumer_tag: Option<String>,

    pub replication_source: String,
    pub replication_destination: String,

    pub pod_name: Option<String>,
    pub export_dir: Option<PathBuf>,
    pub import_file: Option<PathBuf>,

    pub metrics_port: Option<u16>,
}

fn communication_config(config: &Config) -> anyhow::Result<CommunicationMethodConfig> {
    let config = match config.communication_method {
        CommunicationMethodType::Kafka => {
            let brokers = config
                .kafka_brokers
                .clone()
                .context("Missing kafka brokers")?;
            let group_id = config
                .kafka_group_id
                .clone()
                .context("Missing kafka group")?;
            CommunicationMethodConfig::Kafka(KafkaConfig { brokers, group_id })
        }
        CommunicationMethodType::Amqp => {
            let connection_string = config
                .amqp_connection_string
                .clone()
                .context("Missing amqp connection string")?;
            let consumer_tag = config
                .amqp_consumer_tag
                .clone()
                .context("Missing amqp consumer tag")?;
            CommunicationMethodConfig::Amqp(AmqpConfig {
                connection_string,
                consumer_tag,
            })
        }
        CommunicationMethodType::Grpc => CommunicationMethodConfig::Grpc,
    };
    Ok(config)
}

fn replication_config(config: &Config) -> anyhow::Result<Option<ReplicationMethodConfig>> {
    let replication_config = ReplicationMethodConfig {
        queue: match config.communication_method {
            CommunicationMethodType::Kafka => {
                let brokers = config
                    .kafka_brokers
                    .clone()
                    .context("Missing kafka brokers")?;
                let group_id = config
                    .kafka_group_id
                    .clone()
                    .context("Missing kafka group")?;
                CommunicationMethod::Kafka(KafkaConfig { brokers, group_id })
            }
            CommunicationMethodType::Amqp => {
                let connection_string = config
                    .amqp_connection_string
                    .clone()
                    .context("Missing amqp connection string")?;
                let consumer_tag = config
                    .amqp_consumer_tag
                    .clone()
                    .context("Missing amqp consumer tag")?;
                CommunicationMethod::Amqp(AmqpConfig {
                    connection_string,
                    consumer_tag,
                })
            }
            CommunicationMethodType::Grpc => {
                return Ok(None);
            }
        },
        destination: config.replication_destination.clone(),
        source: config.replication_source.clone(),
    };
    Ok(Some(replication_config))
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let config = envy::from_env::<Config>().context("Env vars not set correctly")?;

    let communication_config = communication_config(&config)?;
    let replication_config = replication_config(&config)?;

    status_endpoints::serve();
    metrics::serve(
        config
            .metrics_port
            .unwrap_or_else(|| metrics::DEFAULT_PORT.parse().unwrap()),
    );

    let data_store = SledDatastore::new(&config.db_name).map_err(RegistryError::ConnectionError)?;
    let registry = SchemaRegistryImpl::new(
        data_store,
        config.replication_role,
        communication_config,
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
