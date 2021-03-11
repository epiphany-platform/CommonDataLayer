use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
pub struct ReportServiceConfig {
    /// Kafka topic/AMQP exchange/callback URL to send notifications to (reporting disabled when empty)
    #[structopt(
        name = "report-destination",
        long = "report-destination",
        env = "REPORT_DESTINATION"
    )]
    pub destination: Option<String>,
}
