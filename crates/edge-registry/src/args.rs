use std::str::FromStr;
use structopt::StructOpt;
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
    pub communication_port: u16,
    #[structopt(long, env, default_value = metrics::DEFAULT_PORT)]
    /// Prometheus metrics port
    pub metrics_port: u16,
    #[structopt(flatten)]
    pub consumer_config: ConsumerConfig,
}

#[derive(Clone, Debug, StructOpt)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ConsumerMethod {
    Kafka,
    Amqp,
}

#[derive(Clone, Debug, StructOpt)]
pub struct ConsumerConfig {
    #[structopt(long, env)]
    /// Method of ingestion of messages via Message Queue
    pub method: ConsumerMethod,
    #[structopt(long, env)]
    /// Kafka broker or Amqp (eg. RabbitMQ) host
    pub mq_host: String,
    #[structopt(long, env)]
    /// Kafka group id or Amqp consumer tag
    pub mq_tag: String,
    #[structopt(long, env)]
    /// Kafka topic or Amqp queue
    pub mq_source: String,
}

impl FromStr for ConsumerMethod {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "kafka" => Ok(ConsumerMethod::Kafka),
            "amqp" => Ok(ConsumerMethod::Amqp),
            _ => Err("Invalid consumer method"),
        }
    }
}

impl<'a> From<&'a ConsumerConfig> for CommonConsumerConfig<'a> {
    fn from(config: &'a ConsumerConfig) -> Self {
        match config.method {
            ConsumerMethod::Kafka => CommonConsumerConfig::Kafka {
                brokers: &config.mq_host,
                group_id: &config.mq_tag,
                topic: &config.mq_source,
            },
            ConsumerMethod::Amqp => CommonConsumerConfig::Amqp {
                connection_string: &config.mq_host,
                consumer_tag: &config.mq_tag,
                queue_name: &config.mq_source,
                options: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    mod consumer_method {
        use crate::args::ConsumerMethod;
        use test_case::test_case;

        #[test_case("kafka"    => ConsumerMethod::Kafka ; "kafka all lowercase")]
        #[test_case("Kafka"    => ConsumerMethod::Kafka ; "kafka first uppercase")]
        #[test_case("KAFKA"    => ConsumerMethod::Kafka ; "kafka all uppercase")]
        #[test_case("amqp"     => ConsumerMethod::Amqp  ; "amqp all lowercase")]
        #[test_case("Amqp"     => ConsumerMethod::Amqp  ; "amqp first uppercase")]
        #[test_case("AMQP"     => ConsumerMethod::Amqp  ; "amqp all uppercase")]
        #[test_case("rabbitmq" => panics                ; "invalid string")]
        #[test_case(""         => panics                ; "empty string")]
        fn parses_method_from_string(input: &str) -> ConsumerMethod {
            input.parse().unwrap()
        }
    }
}
