use std::path::PathBuf;

use anyhow::Context;
use structopt::{clap::arg_enum, StructOpt};

use utils::metrics;

pub enum CommunicationMethodConfig {
    Kafka(KafkaConfig),
    Amqp(AmqpConfig),
    Grpc,
}

#[derive(Clone, Debug)]
pub struct KafkaConfig {
    pub brokers: String,
    pub group_id: String,
}

#[derive(Clone, Debug)]
pub struct AmqpConfig {
    pub connection_string: String,
    pub consumer_tag: String,
}

arg_enum! {
    #[derive(Clone, Debug)]
    pub enum CommunicationMethodType {
        Amqp,
        Kafka,
        Grpc,
    }
}

#[derive(StructOpt)]
pub struct Config {
    /// Port to listen on
    #[structopt(long, env)]
    pub input_port: u16,

    /// Postgres username
    #[structopt(long, env = "POSTGRES_USERNAME")]
    pub db_username: String,
    /// Postgres password
    #[structopt(long, env = "POSTGRES_PASSWORD")]
    pub db_password: String,
    /// Host of the postgres server
    #[structopt(long, env = "POSTGRES_HOST")]
    pub db_host: String,
    /// Port on which postgres server listens
    #[structopt(long, env = "POSTGRES_PORT", default_value = "5432")]
    pub db_port: u16,
    /// Database name
    #[structopt(long, env = "POSTGRES_DBNAME")]
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
    #[structopt(long, env, default_value = metrics::DEFAULT_PORT)]
    pub metrics_port: u16,
    /// Port exposing status of the application
    #[structopt(long, default_value = utils::status_endpoints::DEFAULT_PORT, env)]
    pub status_port: u16,
}

pub fn communication_config(config: &Config) -> anyhow::Result<CommunicationMethodConfig> {
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
