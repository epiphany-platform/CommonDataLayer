use anyhow::Context;
use clap::Clap;

#[derive(Clap)]
pub struct Config {
    /// Address of schema registry gRPC API
    #[clap(long, env)]
    pub schema_registry_addr: String,
    /// Address of edge registry gRPC API
    #[clap(long, env)]
    pub edge_registry_addr: String,
    /// Address of object builder gRPC API
    #[clap(long, env)]
    pub object_builder_addr: String,
    /// Address of query router REST API
    #[clap(long, env)]
    pub query_router_addr: String,
    /// Port to listen on
    #[clap(long, env)]
    pub input_port: u16,

    #[clap(flatten)]
    pub communication_method: CommunicationMethodArgs,

    /// Kafka topic/AMQP queue on which API listens for notifications
    #[clap(long, env)]
    pub report_source: Option<String>,
    /// Kafka topic/AMQP exchange/gRPC service address to which API inserts new objects
    #[clap(long, env)]
    pub insert_destination: String,
}

#[derive(Clap)]
pub struct CommunicationMethodArgs {
    /// The method of communication with external services
    #[clap(long, env, arg_enum)]
    communication_method: CommunicationMethod,
    /// Address to Kafka brokers
    #[clap(long, env)]
    kafka_brokers: Option<String>,
    /// Group ID of the Kafka consumer
    #[clap(long, env)]
    kafka_group_id: Option<String>,
    /// Connection URL to AMQP Server
    #[clap(long, env)]
    amqp_connection_string: Option<String>,
    /// AMQP consumer tag
    #[clap(long, env)]
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
            CommunicationMethod::GRpc => CommunicationMethodConfig::Grpc,
        })
    }
}

#[derive(Clap, Clone, Debug)]
enum CommunicationMethod {
    Amqp,
    Kafka,
    #[clap(alias = "grpc")]
    GRpc,
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
