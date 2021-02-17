use anyhow::Context;
use chrono::{DateTime, Utc};
use indradb::SledDatastore;
use rpc::schema_registry::schema_registry_server::SchemaRegistryServer;
use schema_registry::{
    error::RegistryError,
    replication::AmqpConfig,
    replication::MessageQueue,
    replication::{KafkaConfig, MessageQueueConfig, ReplicationRole},
    rpc::SchemaRegistryImpl,
};
use serde::Deserialize;
use std::fs::File;
use std::io::Write;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::PathBuf;
use tonic::transport::Server;
use utils::{metrics, status_endpoints};

enum MessageQueueType {
    Kafka,
    Amqp,
}

impl<'de> Deserialize<'de> for MessageQueueType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?.to_lowercase();
        let mqt = match s.as_str() {
            "kafka" => Self::Kafka,
            "amqp" => Self::Amqp,
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

    pub replication_queue: MessageQueueType,
    pub kafka_brokers: Option<String>,
    pub kafka_group_id: Option<String>,
    pub amqp_connection_string: Option<String>,
    pub amqp_consumer_tag: Option<String>,

    pub replication_topic_or_queue: String,
    pub replication_topic_or_exchange: String,

    pub pod_name: Option<String>,
    pub export_dir: Option<PathBuf>,
    pub import_file: Option<PathBuf>,

    pub metrics_port: Option<u16>,
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let config = envy::from_env::<Config>().context("Env vars not set correctly")?;
    let replication_config = MessageQueueConfig {
        queue: match config.replication_queue {
            MessageQueueType::Kafka => {
                let brokers = config.kafka_brokers.context("Missing kafka brokers")?;
                let group_id = config.kafka_group_id.context("Missing kafka group")?;
                MessageQueue::Kafka(KafkaConfig { brokers, group_id })
            }
            MessageQueueType::Amqp => {
                let connection_string = config
                    .amqp_connection_string
                    .context("Missing amqp connection string")?;
                let consumer_tag = config
                    .amqp_consumer_tag
                    .context("Missing amqp consumer tag")?;
                MessageQueue::Amqp(AmqpConfig {
                    connection_string,
                    consumer_tag,
                })
            }
        },
        topic_or_exchange: config.replication_topic_or_exchange,
        topic_or_queue: config.replication_topic_or_queue,
    };

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
