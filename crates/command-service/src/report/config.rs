use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
pub struct ReportServiceConfig {
    #[structopt(
        name = "report-topic-or-exchange",
        long = "report-topic-or-exchange",
        env = "REPORT_TOPIC_OR_EXCHANGE"
    )]
    pub topic_or_exchange: Option<String>,
}
