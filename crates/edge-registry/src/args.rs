use structopt::{clap::arg_enum, StructOpt};
use utils::communication::consumer::CommonConsumerConfig;
use utils::metrics;

#[derive(Clone, Debug, StructOpt)]
pub struct RegistryConfig {
    #[structopt(long, env)]
    pub postgres_username: String,
    #[structopt(long, env)]
    pub postgres_password: String,
    #[structopt(long, env)]
    pub postgres_host: String,
    #[structopt(long, env, default_value = "5432")]
    pub postgres_port: u16,
    #[structopt(long, env)]
    pub postgres_dbname: String,
    #[structopt(long, env, default_value = "public")]
    pub postgres_schema: String,
    #[structopt(long, env, default_value = "50110")]
    /// gRPC server port to host edge-registry on
    pub rpc_port: u16,
    #[structopt(long, env, default_value = metrics::DEFAULT_PORT)]
    /// Port to listen on for Prometheus requests
    pub metrics_port: u16,
    #[structopt(flatten)]
    pub consumer_config: ConsumerConfig,
}

arg_enum! {
    #[derive(Clone, Debug)]
    #[cfg_attr(test, derive(PartialEq))]
    pub enum ConsumerMethod {
        Kafka,
        Amqp,
    }
}

#[derive(Clone, Debug, StructOpt)]
pub struct ConsumerConfig {
    #[structopt(long, env, possible_values = &ConsumerMethod::variants(), case_insensitive = true)]
    /// Method of ingestion of messages via Message Queue
    pub consumer_method: ConsumerMethod,
    #[structopt(long, env)]
    /// Kafka broker or Amqp (eg. RabbitMQ) host
    pub consumer_host: String,
    #[structopt(long, env)]
    /// Kafka group id or Amqp consumer tag
    pub consumer_tag: String,
    #[structopt(long, env)]
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
