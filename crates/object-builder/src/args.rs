use clap::Clap;
use thiserror::Error;
use utils::communication::consumer::CommonConsumerConfig;

#[derive(Clap, Clone, Debug, Copy)]
pub enum MessageQueue {
    Amqp,
    Kafka,
}

#[derive(Clap, Debug)]
pub struct Args {
    /// The method of ingestion of messages via Message Queue
    #[clap(long, env, arg_enum)]
    pub mq_method: Option<MessageQueue>,
    /// Address of Kafka brokers
    #[clap(long, env)]
    pub kafka_brokers: Option<String>,
    /// Group ID of the Kafka consumer
    #[clap(long, env)]
    pub kafka_group_id: Option<String>,
    /// Connection URL to AMQP server
    #[clap(long, env)]
    pub amqp_connection_string: Option<String>,
    /// AMQP consumer tag
    #[clap(long, env)]
    pub amqp_consumer_tag: Option<String>,
    /// Kafka topic or AMQP queue name
    #[clap(long, env)]
    pub mq_source: Option<String>,
    /// Port to listen on
    #[clap(long, env)]
    pub input_port: u16,
    /// Address of schema registry
    #[clap(long, env)]
    pub schema_registry_addr: String,
    /// Port to listen on for Prometheus requests
    #[clap(long, default_value = utils::metrics::DEFAULT_PORT, env)]
    pub metrics_port: u16,
    /// Port exposing status of the application
    #[clap(long, default_value = utils::status_endpoints::DEFAULT_PORT, env)]
    pub status_port: u16,
}

#[derive(Error, Debug)]
#[error("Missing config variable `{0}`")]
pub struct MissingConfigError(pub &'static str);

impl Args {
    pub fn consumer_config(&self) -> Result<Option<CommonConsumerConfig>, MissingConfigError> {
        match self.mq_method {
            None => Ok(None),
            Some(MessageQueue::Amqp) => {
                let connection_string = self
                    .amqp_connection_string
                    .as_ref()
                    .ok_or(MissingConfigError("AMQP connection string"))?;
                let consumer_tag = self
                    .amqp_consumer_tag
                    .as_ref()
                    .ok_or(MissingConfigError("AMQP consumer tag"))?;
                let queue_name = self
                    .mq_source
                    .as_ref()
                    .ok_or(MissingConfigError("Message queue source"))?;
                let options = Default::default();
                Ok(Some(CommonConsumerConfig::Amqp {
                    connection_string,
                    consumer_tag,
                    queue_name,
                    options,
                }))
            }
            Some(MessageQueue::Kafka) => {
                let brokers = self
                    .kafka_brokers
                    .as_ref()
                    .ok_or(MissingConfigError("Kafka brokers"))?;
                let group_id = self
                    .kafka_group_id
                    .as_ref()
                    .ok_or(MissingConfigError("Kafka group ID"))?;
                let topic = self
                    .mq_source
                    .as_ref()
                    .ok_or(MissingConfigError("Message queue source"))?;
                Ok(Some(CommonConsumerConfig::Kafka {
                    brokers,
                    group_id,
                    topic,
                }))
            }
        }
    }
}
