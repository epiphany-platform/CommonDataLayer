use crate::input::{GRpcConfig, InputConfig, KafkaConfig};
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
        Kafka,
        GRpc,
    }
}

#[derive(Clone, Debug, StructOpt)]
pub struct InputArgs {
    #[structopt(long = "kafka-input-group-id", env = "KAFKA_INPUT_GROUP_ID")]
    pub group_id: Option<String>,
    #[structopt(long = "kafka-input-brokers", env = "KAFKA_INPUT_BROKERS")]
    /// Comma separated list of brokers (eg. host1:9092,host2:9092)
    pub brokers: Option<String>,
    #[structopt(long = "kafka-input-topic", env = "KAFKA_INPUT_TOPIC")]
    pub topic: Option<String>,

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
            IngestionMethod::Kafka => InputConfig::Kafka(KafkaConfig {
                group_id: input_args
                    .group_id
                    .clone()
                    .ok_or(MissingConfigError("Group ID"))?,
                brokers: input_args
                    .brokers
                    .clone()
                    .ok_or(MissingConfigError("Brokers"))?,
                topic: input_args
                    .topic
                    .clone()
                    .ok_or(MissingConfigError("Topic"))?,
                task_limit: input_args.task_limit,
            }),
            IngestionMethod::GRpc => InputConfig::GRpc(GRpcConfig {
                grpc_port: input_args
                    .grpc_port
                    .ok_or(MissingConfigError("GRPC port"))?,
            }),
        })
    }
}
