use anyhow::Context;
use chrono::{DateTime, Utc};
use clap::Clap;
use indradb::SledDatastore;
use rpc::schema_registry::schema_registry_server::SchemaRegistryServer;
use schema_registry::{
    error::RegistryError,
    replication::CommunicationMethod,
    replication::{ReplicationMethodConfig, ReplicationRole},
    rpc::SchemaRegistryImpl,
    AmqpConfig, CommunicationMethodConfig, KafkaConfig,
};
use std::fs::File;
use std::io::Write;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::PathBuf;
use tonic::transport::Server;
use utils::{metrics, status_endpoints};

#[derive(Clap, Clone, Debug)]
pub enum CommunicationMethodType {
    Amqp,
    Kafka,
    #[clap(alias = "grpc")]
    GRpc,
}

#[derive(Clap)]
struct Config {
    /// Port to listen on
    #[clap(long, env)]
    pub input_port: u16,
    /// Database name
    #[clap(long, env)]
    pub db_name: String,
    /// (deprecated)
    #[clap(long, env = "REPLICATION_ROLE", arg_enum)]
    pub replication_role: ReplicationRole,

    /// The method of communication with external services.
    #[clap(long, env = "COMMUNICATION_METHOD", arg_enum)]
    pub communication_method: CommunicationMethodType,
    /// Address of Kafka brokers
    #[clap(long, env)]
    pub kafka_brokers: Option<String>,
    /// Group ID of the consumer
    #[clap(long, env)]
    pub kafka_group_id: Option<String>,
    /// Connection URL to AMQP server
    #[clap(long, env)]
    pub amqp_connection_string: Option<String>,
    /// Consumer tag
    #[clap(long, env)]
    pub amqp_consumer_tag: Option<String>,

    /// Kafka topic/AMQP queue
    #[clap(long, env)]
    pub replication_source: String,
    /// Kafka topic/AMQP exchange
    #[clap(long, env)]
    pub replication_destination: String,

    /// (deprecated) used to promote to `master` role
    #[clap(long, env)]
    pub pod_name: Option<String>,
    /// Directory to save state of the database. The state is saved in newly created folder with timestamp
    #[clap(long, env)]
    pub export_dir: Option<PathBuf>,
    /// JSON file from which SR should load initial state. If the state already exists this env variable witll be ignored
    #[clap(long, env)]
    pub import_file: Option<PathBuf>,
    /// Port to listen on for Prometheus requests
    #[clap(long, env, default_value = metrics::DEFAULT_PORT)]
    /// Port to listen on for Prometheus requests
    pub metrics_port: u16,
    /// Port exposing status of the application
    #[clap(long, default_value = utils::status_endpoints::DEFAULT_PORT, env)]
    pub status_port: u16,
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
        CommunicationMethodType::GRpc => CommunicationMethodConfig::Grpc,
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
            CommunicationMethodType::GRpc => {
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
    utils::set_aborting_panic_hook();
    utils::tracing::init();
    let config = Config::parse();

    let communication_config = communication_config(&config)?;
    let replication_config = replication_config(&config)?;

    status_endpoints::serve(config.status_port);
    metrics::serve(config.metrics_port);

    let data_store = SledDatastore::new(&config.db_name)
        .map_err(|err| anyhow::anyhow!("{}", RegistryError::ConnectionError(err)))?;
    let registry = SchemaRegistryImpl::new(
        data_store,
        config.replication_role,
        communication_config,
        replication_config,
        config.pod_name,
    )
    .await?;

    if let Some(export_dir_path) = config.export_dir {
        let exported = registry
            .export_all()
            .map_err(|err| anyhow::anyhow!("{}", err))?;
        let exported = serde_json::to_string(&exported)?;
        let export_path = export_path(export_dir_path);
        let mut file = File::create(export_path)?;
        write!(file, "{}", exported)?;
    }

    if let Some(import_path) = config.import_file {
        let imported = File::open(import_path).map_err(|err| anyhow::anyhow!("{}", err))?;
        let imported = serde_json::from_reader(imported)?;
        registry
            .import_all(imported)
            .map_err(|err| anyhow::anyhow!("{}", err))?;
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
