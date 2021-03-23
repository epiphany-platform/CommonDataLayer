use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
pub struct ReportServiceConfig {
    #[structopt(
        name = "report-topic-or-exchange",
        long = "report-topic-or-exchange",
        env = "REPORT_DESTINATION"
    )]
    pub destination: Option<String>,
}
