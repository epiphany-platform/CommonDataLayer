pub use crate::communication::error::Error;
pub use crate::communication::message::GenericMessage;
use crate::communication::resolution::Resolution;
use crate::output::OutputPlugin;
use crate::report::ReportService;
use std::sync::Arc;

mod error;
mod message;

pub mod resolution;

pub struct MessageRouter<P: OutputPlugin> {
    report_service: Arc<ReportService>,
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
    pub fn new(report_service: ReportService, output_plugin: P) -> Self {
        Self {
            report_service: Arc::new(report_service),
            output_plugin: Arc::new(output_plugin),
        }
    }

    pub async fn handle_message(&self, msg: GenericMessage) -> Result<(), Error> {
        let status = self
            .output_plugin
            .handle_message(msg)
            .await
            .map_err(Error::FailedToInsertData)?;

        match status {
            Resolution::StorageLayerFailure {
                ref description,
                ref object_id,
            } => {
                self.report_service
                    .report_failure("TODO", &description, *object_id)
                    .await
                    .map_err(Error::ReportingError)?;
            }
            Resolution::UserFailure {
                ref description,
                ref object_id,
                ref context,
            } => {
                self.report_service
                    .report_failure(
                        "TODO",
                        &format!("{}; caused by `{}`", description, context),
                        *object_id,
                    )
                    .await
                    .map_err(Error::ReportingError)?;
            }
            Resolution::CommandServiceFailure { ref object_id } => {
                self.report_service
                    .report_failure("TODO", "Internal server error", *object_id)
                    .await
                    .map_err(Error::ReportingError)?;
            }
            Resolution::Success => {}
        }

        Ok(())
    }
}
