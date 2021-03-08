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
    pub communication_method: CommunicationMethodArgs,

    #[structopt(long, env)]
    pub report_source: String,
    #[structopt(long, env)]
    pub data_router_source: String,
}

#[derive(StructOpt)]
pub struct CommunicationMethodArgs {
    #[structopt(long, env, possible_values = &CommunicationMethod::variants(), case_insensitive = true)]
    communication_method: CommunicationMethod,
    #[structopt(long, env)]
    kafka_brokers: Option<String>,
    #[structopt(long, env)]
    kafka_group_id: Option<String>,
    #[structopt(long, env)]
    amqp_connection_string: Option<String>,
    #[structopt(long, env)]
    amqp_consumer_tag: Option<String>,
}

impl CommunicationMethodArgs {
    pub fn config(&self) -> anyhow::Result<CommunicationMethodConfig> {
        Ok(match self.communication_method {
            CommunicationMethod::Kafka => {
                let brokers = self
                    .kafka_brokers
                    .clone()
                    .context("Missing kafka brokers")?;
                let group_id = self.kafka_group_id.clone().context("Missing kafka group")?;
                CommunicationMethodConfig::Kafka { group_id, brokers }
            }
            CommunicationMethod::Amqp => {
                let connection_string = self
                    .amqp_connection_string
                    .clone()
                    .context("Missing AMQP connection string")?;
                let consumer_tag = self
                    .amqp_consumer_tag
                    .clone()
                    .context("Missing AMQP consumer tag")?;
                CommunicationMethodConfig::Amqp {
                    connection_string,
                    consumer_tag,
                }
            }
            CommunicationMethod::Grpc => CommunicationMethodConfig::Grpc,
        })
    }
}

arg_enum! {
    #[derive(Clone, Debug)]
    enum CommunicationMethod {
        Amqp,
        Kafka,
        Grpc
    }
}

#[derive(Clone, Debug)]
pub enum CommunicationMethodConfig {
    Kafka {
        group_id: String,
        brokers: String,
    },
    Amqp {
        connection_string: String,
        consumer_tag: String,
    },
    Grpc,
}
