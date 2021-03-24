use std::fs::File;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::PathBuf;

use anyhow::Context;
use structopt::{clap::arg_enum, StructOpt};
use tonic::transport::Server;
use utils::{metrics, status_endpoints};

use rpc::schema_registry::schema_registry_server::SchemaRegistryServer;
use schema_registry::rpc::SchemaRegistryImpl;
use schema_registry::{AmqpConfig, CommunicationMethodConfig, KafkaConfig};

arg_enum! {
    #[derive(Clone, Debug)]
    pub enum CommunicationMethodType {
        Amqp,
        Kafka,
        Grpc,
    }
}

#[derive(StructOpt)]
struct Config {
    /// Port to listen on
    #[structopt(long, env)]
    pub input_port: u16,
    /// Database name
    #[structopt(long, env)]
    pub db_name: String,

    /// The method of communication with external services.
    #[structopt(long, env = "COMMUNICATION_METHOD", possible_values = &CommunicationMethodType::variants(), case_insensitive = true)]
    pub communication_method: CommunicationMethodType,
    /// Address of Kafka brokers
    #[structopt(long, env)]
    pub kafka_brokers: Option<String>,
    /// Group ID of the consumer
    #[structopt(long, env)]
    pub kafka_group_id: Option<String>,
    /// Connection URL to AMQP server
    #[structopt(long, env)]
    pub amqp_connection_string: Option<String>,
    /// Consumer tag
    #[structopt(long, env)]
    pub amqp_consumer_tag: Option<String>,

    /// Directory to save state of the database. The state is saved in newly created folder with timestamp
    #[structopt(long, env)]
    pub export_dir: Option<PathBuf>,
    /// JSON file from which SR should load initial state. If the state already exists this env variable witll be ignored
    #[structopt(long, env)]
    pub import_file: Option<PathBuf>,
    /// Port to listen on for Prometheus requests
    #[structopt(long, env)]
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
