use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
pub struct DruidOutputConfig {
    #[structopt(
        name = "druid-brokers",
        long = "druid-output-brokers",
        env = "DRUID_OUTPUT_BROKERS"
    )]
    pub brokers: String,
    #[structopt(
        name = "druid-topic",
        long = "druid-output-topic",
        env = "DRUID_OUTPUT_TOPIC"
    )]
    pub topic: String,
}
