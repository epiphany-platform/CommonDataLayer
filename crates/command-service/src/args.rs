use crate::communication::config::{CommunicationConfig, GRpcConfig, MessageQueueConfig};
use crate::output::OutputArgs;
use crate::report::ReportServiceConfig;
use structopt::clap::arg_enum;
use structopt::StructOpt;
use thiserror::Error;
use utils::metrics;

#[derive(Clone, Debug, StructOpt)]
pub struct Args {
    #[structopt(flatten)]
    communication_args: CommunicationArgs,

    #[structopt(flatten)]
    pub output_config: OutputArgs,

    #[structopt(flatten)]
    pub report_config: ReportServiceConfig,

    #[structopt(default_value = metrics::DEFAULT_PORT, env)]
    pub metrics_port: u16,
}

arg_enum! {
    #[derive(Clone, Debug)]
    pub enum CommunicationMethod {
        Amqp,
        Kafka,
        GRpc,
    }
}

#[derive(Clone, Debug, StructOpt)]
pub struct CommunicationArgs {
    #[structopt(long, env = "COMMUNICATION_METHOD", possible_values = &CommunicationMethod::variants(), case_insensitive = true)]
    pub communication_method: CommunicationMethod,

    #[structopt(long, env)]
    pub kafka_brokers: Option<String>,
    #[structopt(long, env)]
    pub kafka_group_id: Option<String>,
    #[structopt(long, env)]
    pub amqp_connection_string: Option<String>,
    #[structopt(long, env)]
    pub amqp_consumer_tag: Option<String>,

    #[structopt(long = "rpc-input-port", env = "RPC_PORT")]
    pub grpc_port: Option<u16>,
    #[structopt(long, env)]
    pub ordered_topics_or_queues: Option<String>,
    #[structopt(long, env)]
    pub unordered_topics_or_queues: Option<String>,

    #[structopt(
        long = "threaded-task-limit",
        env = "THREADED_TASK_LIMIT",
        default_value = "32"
    )]
    /// Amount of tasks that can be spawned, and process data input, at one given time
    pub task_limit: usize,
}

#[derive(Error, Debug)]
#[error("Missing config variable `{0}`")]
pub struct MissingConfigError(pub &'static str);

impl Args {
    pub fn communication_config(&self) -> Result<CommunicationConfig, MissingConfigError> {
        let communication_args = &self.communication_args;
        Ok(match communication_args.communication_method {
            CommunicationMethod::Amqp | CommunicationMethod::Kafka => {
                let ordered_topics_or_queues: Vec<_> = communication_args
                    .ordered_topics_or_queues
                    .clone()
                    .unwrap_or_default()
                    .split(',')
                    .filter(|queue_name| !queue_name.is_empty())
                    .map(String::from)
                    .collect();
                let unordered_topics_or_queues: Vec<_> = communication_args
                    .unordered_topics_or_queues
                    .clone()
                    .unwrap_or_default()
                    .split(',')
                    .filter(|queue_name| !queue_name.is_empty())
                    .map(String::from)
                    .collect();

                let task_limit = communication_args.task_limit;

                if ordered_topics_or_queues.is_empty() && unordered_topics_or_queues.is_empty() {
                    return Err(MissingConfigError("Queue names"));
                }

                let config = match communication_args.communication_method {
                    CommunicationMethod::Amqp => {
                        let consumer_tag = communication_args
                            .amqp_consumer_tag
                            .clone()
                            .ok_or(MissingConfigError("AMQP consumer tag"))?;
                        let connection_string =
                            communication_args
                                .amqp_connection_string
                                .clone()
                                .ok_or(MissingConfigError("AMQP connection string"))?;
                        MessageQueueConfig::Amqp {
                            connection_string,
                            consumer_tag,
                            ordered_queue_names: ordered_topics_or_queues,
                            unordered_queue_names: unordered_topics_or_queues,
                            task_limit,
                        }
                    }
                    CommunicationMethod::Kafka => {
                        let brokers = communication_args
                            .kafka_brokers
                            .clone()
                            .ok_or(MissingConfigError("Kafka brokers"))?;
                        let group_id = communication_args
                            .kafka_group_id
                            .clone()
                            .ok_or(MissingConfigError("Kafka group"))?;
                        MessageQueueConfig::Kafka {
                            brokers,
                            group_id,
                            ordered_topics: ordered_topics_or_queues,
                            unordered_topics: unordered_topics_or_queues,
                            task_limit,
                        }
                    }
                    _ => unreachable!(),
                };

                CommunicationConfig::MessageQueue(config)
            }
            CommunicationMethod::GRpc => CommunicationConfig::GRpc(GRpcConfig {
                grpc_port: communication_args
                    .grpc_port
                    .ok_or(MissingConfigError("GRPC port"))?,
            }),
        })
    }
}
