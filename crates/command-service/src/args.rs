use crate::communication::config::CommunicationConfig;
use crate::output::OutputArgs;
use crate::report::ReportServiceConfig;
use structopt::clap::arg_enum;
use structopt::StructOpt;
use thiserror::Error;
use url::Url;
use utils::metrics;

#[derive(Clone, Debug, StructOpt)]
pub struct Args {
    #[structopt(flatten)]
    communication_args: CommunicationArgs,

    #[structopt(flatten)]
    pub output_config: OutputArgs,

    #[structopt(flatten)]
    pub report_config: ReportServiceConfig,

    /// Port to listen on for Prometheus requests
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
    /// The method of communication with external services
    #[structopt(long, env = "COMMUNICATION_METHOD", possible_values = &CommunicationMethod::variants(), case_insensitive = true)]
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
    #[structopt(long = "rpc-input-port", env = "GRPC_PORT")]
    pub grpc_port: Option<u16>,
    /// URL to send notifications to. Used with `grpc` communication method
    #[structopt(long, env)]
    pub report_endpoint_url: Option<Url>,

    /// Kafka topics/AMQP queues with ordered messages. Ignored with `grpc` communication method
    #[structopt(long, env)]
    pub ordered_sources: Option<String>,
    /// Kafka topics/AMQP queues with unordered messages. Ignored with `grpc` communication method
    #[structopt(long, env)]
    pub unordered_sources: Option<String>,

    /// Max requests handled in parallel
    #[structopt(
        long = "threaded-task-limit",
        env = "THREADED_TASK_LIMIT",
        default_value = "32"
    )]
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
                let ordered_sources: Vec<_> = communication_args
                    .ordered_sources
                    .clone()
                    .unwrap_or_default()
                    .split(',')
                    .filter(|queue_name| !queue_name.is_empty())
                    .map(String::from)
                    .collect();
                let unordered_sources: Vec<_> = communication_args
                    .unordered_sources
                    .clone()
                    .unwrap_or_default()
                    .split(',')
                    .filter(|queue_name| !queue_name.is_empty())
                    .map(String::from)
                    .collect();

                let task_limit = communication_args.task_limit;

                if ordered_sources.is_empty() && unordered_sources.is_empty() {
                    return Err(MissingConfigError("Queue names"));
                }

                match communication_args.communication_method {
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
                        CommunicationConfig::Amqp {
                            connection_string,
                            consumer_tag,
                            ordered_queue_names: ordered_sources,
                            unordered_queue_names: unordered_sources,
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
                        CommunicationConfig::Kafka {
                            brokers,
                            group_id,
                            ordered_topics: ordered_sources,
                            unordered_topics: unordered_sources,
                            task_limit,
                        }
                    }
                    _ => unreachable!(),
                }
            }
            CommunicationMethod::GRpc => CommunicationConfig::Grpc {
                grpc_port: communication_args
                    .grpc_port
                    .ok_or(MissingConfigError("GRPC port"))?,
                report_endpoint_url: communication_args
                    .clone()
                    .report_endpoint_url
                    .ok_or(MissingConfigError("Report endpoint url"))?,
            },
        })
    }
}
