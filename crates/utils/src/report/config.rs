use clap::Clap;

#[derive(Clone, Debug, Clap)]
pub struct ReportServiceConfig {
    /// Kafka topic/AMQP exchange/callback URL to send notifications to (reporting disabled when empty)
    #[clap(
        name = "report-destination",
        long = "report-destination",
        env = "REPORT_DESTINATION"
    )]
    pub destination: Option<String>,
}
