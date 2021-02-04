use anyhow::Context;
use structopt::{clap::arg_enum, StructOpt};

#[derive(StructOpt)]
pub struct Config {
    #[structopt(long, env)]
    pub schema_registry_addr: String,
    #[structopt(long, env)]
    pub query_router_addr: String,
    #[structopt(long, env)]
    pub input_port: u16,

    #[structopt(flatten)]
    pub message_queue: MessageQueueArgs,

    #[structopt(long, env)]
    pub report_topic_or_queue: String,
    #[structopt(long, env)]
    pub data_router_topic_or_queue: String,
}

#[derive(StructOpt)]
pub struct MessageQueueArgs {
    #[structopt(long, env, possible_values = &MessageQueue::variants(), case_insensitive = true)]
    message_queue: MessageQueue,
    #[structopt(long, env)]
    kafka_brokers: Option<String>,
    #[structopt(long, env)]
    kafka_group_id: Option<String>,
    #[structopt(long, env)]
    amqp_connection_string: Option<String>,
    #[structopt(long, env)]
    amqp_consumer_tag: Option<String>,
}

impl MessageQueueArgs {
    pub fn config(&self) -> anyhow::Result<MessageQueueConfig> {
        Ok(match self.message_queue {
            MessageQueue::Kafka => {
                let brokers = self
                    .kafka_brokers
                    .clone()
                    .context("Missing kafka brokers")?;
                let group_id = self.kafka_group_id.clone().context("Missing kafka group")?;
                MessageQueueConfig::Kafka { group_id, brokers }
            }
            MessageQueue::Amqp => {
                let connection_string = self
                    .amqp_connection_string
                    .clone()
                    .context("Missing AMQP connection string")?;
                let consumer_tag = self
                    .amqp_consumer_tag
                    .clone()
                    .context("Missing AMQP consumer tag")?;
                MessageQueueConfig::Amqp {
                    connection_string,
                    consumer_tag,
                }
            }
        })
    }
}

arg_enum! {
    #[derive(Clone, Debug)]
    enum MessageQueue {
        Amqp,
        Kafka,
    }
}

#[derive(Clone, Debug)]
pub enum MessageQueueConfig {
    Kafka {
        group_id: String,
        brokers: String,
    },
    Amqp {
        connection_string: String,
        consumer_tag: String,
    },
}
