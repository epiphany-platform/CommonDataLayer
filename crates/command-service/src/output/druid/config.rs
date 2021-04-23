use clap::Clap;

#[derive(Clone, Debug, Clap)]
pub struct DruidOutputConfig {
    /// Kafka brokers
    #[clap(
        name = "druid-brokers",
        long = "druid-output-brokers",
        env = "DRUID_OUTPUT_BROKERS"
    )]
    pub brokers: String,
    /// Kafka topic
    #[clap(
        name = "druid-topic",
        long = "druid-output-topic",
        env = "DRUID_OUTPUT_TOPIC"
    )]
    pub topic: String,
}
