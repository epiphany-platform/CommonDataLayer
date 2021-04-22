use crate::message_types::BorrowedInsertMessage;
use crate::report::error::Error;
pub use config::ReportServiceConfig;
pub use full_report_sender::{FullReportSender, FullReportSenderBase};

mod config;
pub mod error;
mod full_report_sender;

#[derive(Clone)]
pub enum ReportSender {
    Full(FullReportSenderBase),
    Disabled,
}

#[async_trait::async_trait]
pub trait Reporter: Send + Sync + 'static {
    async fn report(self: Box<Self>, description: &str) -> Result<(), Error>;
}

#[async_trait::async_trait]
impl Reporter for () {
    async fn report(self: Box<Self>, _: &str) -> Result<(), Error> {
        Ok(())
    }
}

impl ReportSender {
    pub fn with_message_body(self, msg: &BorrowedInsertMessage) -> Box<dyn Reporter> {
        match self {
            ReportSender::Full(config) => Box::new(FullReportSender {
                producer: config.publisher,
                destination: config.destination,
                output_plugin: config.output_plugin,
                msg: msg.to_owned(),
            }),
            ReportSender::Disabled => Box::new(()),
        }
    }
}
