pub use crate::communication::error::Error;
pub use crate::communication::message::GenericMessage;
use crate::communication::resolution::Resolution;
use crate::output::OutputPlugin;
use crate::report::ReportSender;
use std::sync::Arc;

mod error;
mod message;

pub mod resolution;

pub struct MessageRouter<P: OutputPlugin> {
    report_service: Arc<ReportSender>,
    output_plugin: Arc<P>,
}

impl<P: OutputPlugin> Clone for MessageRouter<P> {
    fn clone(&self) -> Self {
        MessageRouter {
            report_service: self.report_service.clone(),
            output_plugin: self.output_plugin.clone(),
        }
    }
}

impl<P: OutputPlugin> MessageRouter<P> {
    pub fn new(report_service: ReportSender, output_plugin: P) -> Self {
        Self {
            report_service: Arc::new(report_service),
            output_plugin: Arc::new(output_plugin),
        }
    }

    pub async fn handle_message(&self, msg: GenericMessage) -> Result<(), Error> {
        let mut instance = self.report_service.with_message_body(&msg);

        let status = self.output_plugin.handle_message(msg).await;

        let description = match status {
            Resolution::StorageLayerFailure { description } => description,
            Resolution::CommandServiceFailure => "Internal service error".to_string(),
            Resolution::UserFailure {
                description,
                context,
            } => format!("{}; caused by `{}`", description, context),
            Resolution::Success => "Success".to_string(),
        };

        instance
            .report(&description)
            .await
            .map_err(Error::ReportingError)?;

        Ok(())
    }
}
