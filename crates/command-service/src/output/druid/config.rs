use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
pub struct DruidOutputConfig {
    /// Kafka brokers
    #[structopt(
        name = "druid-brokers",
        long = "druid-output-brokers",
        env = "DRUID_OUTPUT_BROKERS"
    )]
    pub brokers: String,
    /// Kafka topic
    #[structopt(
        name = "druid-topic",
        long = "druid-output-topic",
        env = "DRUID_OUTPUT_TOPIC"
    )]
    pub topic: String,
}
