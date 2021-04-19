use clap::{ArgEnum, Clap};
use std::str::FromStr;
use utils::communication::consumer::CommonConsumerConfig;
use utils::metrics;

#[derive(Clone, Debug, Clap)]
pub struct RegistryConfig {
    #[clap(long, env)]
    pub postgres_username: String,
    #[clap(long, env)]
    pub postgres_password: String,
    #[clap(long, env)]
    pub postgres_host: String,
    #[clap(long, env, default_value = "5432")]
    pub postgres_port: u16,
    #[clap(long, env)]
    pub postgres_dbname: String,
    #[clap(long, env, default_value = "public")]
    pub postgres_schema: String,
    #[clap(long, env, default_value = "50110")]
    /// gRPC server port to host edge-registry on
    pub rpc_port: u16,
    #[clap(long, env, default_value = metrics::DEFAULT_PORT)]
    /// Port to listen on for Prometheus requests
    pub metrics_port: u16,
    /// Port exposing status of the application
    #[clap(long, default_value = utils::status_endpoints::DEFAULT_PORT, env)]
    pub status_port: u16,
    #[clap(flatten)]
    pub consumer_config: ConsumerConfig,
}

#[derive(Clone, Debug, ArgEnum)]
pub enum ConsumerMethod {
    Kafka,
    Amqp,
}

impl FromStr for ConsumerMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ArgEnum::from_str(s, true)
    }
}

#[derive(Clone, Debug, Clap)]
pub struct ConsumerConfig {
    #[clap(long, env, case_insensitive = true)]
    /// Method of ingestion of messages via Message Queue
    pub consumer_method: ConsumerMethod,
    #[clap(long, env)]
    /// Kafka broker or Amqp (eg. RabbitMQ) host
    pub consumer_host: String,
    #[clap(long, env)]
    /// Kafka group id or Amqp consumer tag
    pub consumer_tag: String,
    #[clap(long, env)]
    /// Kafka topic or Amqp queue
    pub consumer_source: String,
}

impl<'a> From<&'a ConsumerConfig> for CommonConsumerConfig<'a> {
    fn from(config: &'a ConsumerConfig) -> Self {
        match config.consumer_method {
            ConsumerMethod::Kafka => CommonConsumerConfig::Kafka {
                brokers: &config.consumer_host,
                group_id: &config.consumer_tag,
                topic: &config.consumer_source,
            },
            ConsumerMethod::Amqp => CommonConsumerConfig::Amqp {
                connection_string: &config.consumer_host,
                consumer_tag: &config.consumer_tag,
                queue_name: &config.consumer_source,
                options: None,
            },
        }
    }
}
