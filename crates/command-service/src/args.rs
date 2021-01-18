use crate::input::{GRpcConfig, InputConfig, MessageQueueConfig};
use crate::output::OutputArgs;
use crate::report::ReportServiceConfig;
use structopt::clap::arg_enum;
use structopt::StructOpt;
use thiserror::Error;

#[derive(Clone, Debug, StructOpt)]
pub struct Args {
    #[structopt(flatten)]
    pub output_config: OutputArgs,

    #[structopt(long, env = "INGESTION_METHOD", possible_values = &IngestionMethod::variants(), case_insensitive = true)]
    ingestion_method: IngestionMethod,

    #[structopt(flatten)]
    input_args: InputArgs,

    #[structopt(flatten)]
    pub report_config: ReportServiceConfig,
}

arg_enum! {
    #[derive(Clone, Debug)]
    pub enum IngestionMethod {
        MessageQueue,
        GRpc,
    }
}

#[derive(Clone, Debug, StructOpt)]
pub struct InputArgs {
    #[structopt(long = "queue-consumer-tag", env = "QUEUE_CONSUMER_TAG")]
    pub consumer_tag: Option<String>,
    #[structopt(long = "queue-connection-string", env = "QUEUE_CONNECTION_STRING")]
    pub connection_string: Option<String>,
    #[structopt(long = "ordered-queue-names", env = "ORDERED_QUEUE_NAMES")]
    pub ordered_queue_names: Option<String>,
    #[structopt(long = "unordered-queue-names", env = "UNORDERED_QUEUE_NAMES")]
    pub unordered_queue_names: Option<String>,

    #[structopt(
        long = "threaded-task-limit",
        env = "THREADED_TASK_LIMIT",
        default_value = "32"
    )]
    /// Amount of tasks that can be spawned, and process data input, at one given time
    pub task_limit: usize,

    #[structopt(long = "rpc-input-port", env = "RPC_PORT")]
    pub grpc_port: Option<u16>,
}

#[derive(Error, Debug)]
#[error("Missing config variable `{0}`")]
pub struct MissingConfigError(&'static str);

impl Args {
    pub fn input_config(&self) -> Result<InputConfig, MissingConfigError> {
        let input_args = &self.input_args;
        Ok(match self.ingestion_method {
            IngestionMethod::MessageQueue => {
                let consumer_tag = input_args
                    .consumer_tag
                    .clone()
                    .ok_or(MissingConfigError("Consumer tag"))?;
                let connection_string = input_args
                    .connection_string
                    .clone()
                    .ok_or(MissingConfigError("Connection string"))?;
                let ordered_queue_names: Vec<_> = input_args
                    .ordered_queue_names
                    .clone()
                    .unwrap_or_default()
                    .split(',')
                    .filter(|queue_name| !queue_name.is_empty())
                    .map(String::from)
                    .collect();
                let unordered_queue_names: Vec<_> = input_args
                    .unordered_queue_names
                    .clone()
                    .unwrap_or_default()
                    .split(',')
                    .filter(|queue_name| !queue_name.is_empty())
                    .map(String::from)
                    .collect();

                let task_limit = input_args.task_limit;

                if ordered_queue_names.is_empty() && unordered_queue_names.is_empty() {
                    return Err(MissingConfigError("Queue names"));
                }
                InputConfig::MessageQueue(MessageQueueConfig {
                    connection_string,
                    consumer_tag,
                    ordered_queue_names,
                    task_limit,
                    unordered_queue_names,
                })
            }
            IngestionMethod::GRpc => InputConfig::GRpc(GRpcConfig {
                grpc_port: input_args
                    .grpc_port
                    .ok_or(MissingConfigError("GRPC port"))?,
            }),
        })
    }
}
