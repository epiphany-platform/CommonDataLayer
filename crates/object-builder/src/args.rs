use std::net::{Ipv4Addr, SocketAddrV4};
use structopt::{clap::arg_enum, StructOpt};
use thiserror::Error;
use utils::communication::parallel_consumer::ParallelCommonConsumerConfig;

arg_enum! {
    #[derive(Clone, Debug)]
    pub enum CommunicationMethod {
        Amqp,
        Kafka,
        GRpc,
    }
}

#[derive(StructOpt, Debug)]
pub struct Args {
    /// The method of communication with external services
    #[structopt(long, env, possible_values = &CommunicationMethod::variants(), case_insensitive = true)]
    pub communication_method: CommunicationMethod,
    /// Address of Kafka brokers
    #[structopt(long, env)]
    pub kafka_brokers: Option<String>,
    /// Group ID of the Kafka consumer
    #[structopt(long, env)]
    pub kafka_group_id: Option<String>,
    /// Connection URL to AMQP server
    #[structopt(long, env)]
    pub amqp_connection_string: Option<String>,
    /// AMQP consumer tag
    #[structopt(long, env)]
    pub amqp_consumer_tag: Option<String>,
    /// Port to listen on
    #[structopt(long, env)]
    pub grpc_port: Option<u16>,
    /// Address of schema registry
    #[structopt(long, env)]
    pub schema_registry_addr: String,
    /// Port to listen on for Prometheus requests
    #[structopt(long, default_value = utils::metrics::DEFAULT_PORT, env)]
    pub metrics_port: u16,
}

#[derive(Error, Debug)]
#[error("Missing config variable `{0}`")]
pub struct MissingConfigError(pub &'static str);

impl Args {
    pub fn communication_config(&self) -> Result<ParallelCommonConsumerConfig, MissingConfigError> {
        match self.communication_method {
            CommunicationMethod::Amqp => {
                todo!()
            }
            CommunicationMethod::Kafka => {
                todo!()
            }
            CommunicationMethod::GRpc => {
                let port = self.grpc_port.ok_or(MissingConfigError("Grpc port"))?;
                let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port);
                Ok(ParallelCommonConsumerConfig::Grpc { addr })
            }
        }
    }
}
