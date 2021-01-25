use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Config {
    #[structopt(long = "schema-registry-addr", env = "SCHEMA_REGISTRY_ADDR")]
    pub registry_addr: String,
    #[structopt(long = "query-router-addr", env = "QUERY_ROUTER_ADDR")]
    pub query_router_addr: String,
    #[structopt(long = "input-port", env = "INPUT_PORT")]
    pub input_port: u16,

    #[structopt(flatten)]
    pub kafka: KafkaConfig,

    #[structopt(long = "report-topic", env = "REPORT_TOPIC")]
    pub report_topic: String,
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
