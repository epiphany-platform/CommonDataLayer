use crate::communication::GenericMessage;
pub use config::ReportServiceConfig;
pub use error::Error;
pub use verbose_report_service::{VerboseReportServiceConfig, VerboseReportServiceInstance};

mod config;
mod error;
mod verbose_report_service;

pub enum ReportService {
    Verbose(VerboseReportServiceConfig),
    Disabled,
}

#[async_trait::async_trait]
pub trait ReportServiceInstance: Send + Sync + 'static {
    async fn report(&self, description: &str) -> Result<(), Error>;
}

#[async_trait::async_trait]
impl ReportServiceInstance for () {
    async fn report(&self, _: &str) -> Result<(), Error> {
        Ok(())
    }
}

impl ReportService {
    pub fn instantiate(&self, msg: &GenericMessage) -> Box<dyn ReportServiceInstance> {
        match self {
            ReportService::Verbose(config) => Box::new(VerboseReportServiceInstance {
                producer: config.producer.clone(),
                topic: config.topic.clone(),
                output_plugin: config.output_plugin.clone(),
                msg: msg.clone(),
            }),
            ReportService::Disabled => Box::new(()),
        }
    }
}
