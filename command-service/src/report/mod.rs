use crate::communication::GenericMessage;
pub use config::ReportServiceConfig;
pub use error::Error;
pub use full_report_service::{FullReportServiceConfig, FullReportServiceInstance};

mod config;
mod error;
mod full_report_service;

pub enum ReportService {
    Full(FullReportServiceConfig),
    Disabled,
}

#[async_trait::async_trait]
pub trait ReportServiceInstance: Send + Sync + 'static {
    async fn report(&mut self, description: &str) -> Result<(), Error>;
}

#[async_trait::async_trait]
impl ReportServiceInstance for () {
    async fn report(&mut self, _: &str) -> Result<(), Error> {
        Ok(())
    }
}

impl ReportService {
    pub fn instantiate(&self, msg: &GenericMessage) -> Box<dyn ReportServiceInstance> {
        match self {
            ReportService::Full(config) => Box::new(FullReportServiceInstance {
                producer: config.producer.clone(),
                topic: config.topic.clone(),
                output_plugin: config.output_plugin.clone(),
                msg: msg.clone(),
            }),
            ReportService::Disabled => Box::new(()),
        }
    }
}
