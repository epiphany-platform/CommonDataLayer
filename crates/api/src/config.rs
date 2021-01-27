use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Config {
    #[structopt(long, env)]
    pub schema_registry_addr: String,
    #[structopt(long, env)]
    pub query_router_addr: String,
    #[structopt(long, env)]
    pub input_port: u16,

    #[structopt(flatten)]
    pub kafka: KafkaConfig,

    #[structopt(long, env)]
    pub report_topic: String,
    #[structopt(long, env)]
    pub data_router_topic: String,
}

#[derive(StructOpt)]
pub struct KafkaConfig {
    #[structopt(
        long = "kafka-group-id",
        env = "KAFKA_GROUP_ID",
        default_value = "cdl-api"
    )]
    pub group_id: String,
    #[structopt(
        long = "kafka-brokers",
        env = "KAFKA_BROKERS",
        default_value = "localhost:9092"
    )]
    pub brokers: String,
}
