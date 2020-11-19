use crate::communication::GenericMessage;
pub use config::ReportServiceConfig;
pub use error::Error;
pub use full_report_sender::{FullReportSender, FullReportSenderBase};

mod config;
mod error;
mod full_report_sender;

pub enum ReportSender {
    Full(FullReportSenderBase),
    Disabled,
}

#[async_trait::async_trait]
pub trait Reporter: Send + Sync + 'static {
    async fn report(&mut self, description: &str) -> Result<(), Error>;
}

#[async_trait::async_trait]
impl Reporter for () {
    async fn report(&mut self, _: &str) -> Result<(), Error> {
        Ok(())
    }
}

impl ReportSender {
    pub fn with_message_body(&self, msg: &GenericMessage) -> Box<dyn Reporter> {
        match self {
            ReportSender::Full(config) => Box::new(FullReportSender {
                producer: config.producer.clone(),
                topic: config.topic.clone(),
                output_plugin: config.output_plugin.clone(),
                msg: msg.clone(),
            }),
            ReportSender::Disabled => Box::new(()),
        }
    }
}
