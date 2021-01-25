use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
pub struct ReportServiceConfig {
    #[structopt(name = "report-broker", long = "report-broker", env = "REPORT_BROKER")]
    pub broker: Option<String>,
    #[structopt(name = "report-topic", long = "report-topic", env = "REPORT_TOPIC")]
    pub topic: Option<String>,
}
