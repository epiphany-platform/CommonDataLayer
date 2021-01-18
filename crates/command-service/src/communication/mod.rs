use crate::communication::resolution::Resolution;
use crate::output::OutputPlugin;
use crate::report::{Error, ReportSender};
use std::sync::Arc;
use utils::message_types::BorrowedInsertMessage;

pub mod resolution;

pub struct MessageRouter<P: OutputPlugin> {
    report_sender: ReportSender,
    output_plugin: Arc<P>,
}

impl<P: OutputPlugin> Clone for MessageRouter<P> {
    fn clone(&self) -> Self {
        MessageRouter {
            report_sender: self.report_sender.clone(),
            output_plugin: self.output_plugin.clone(),
        }
    }
}

impl<P: OutputPlugin> MessageRouter<P> {
    pub fn new(report_sender: ReportSender, output_plugin: P) -> Self {
        Self {
            report_sender,
            output_plugin: Arc::new(output_plugin),
        }
    }

    pub async fn handle_message(&self, msg: BorrowedInsertMessage<'_>) -> Result<(), Error> {
        let instance = self.report_sender.clone().with_message_body(&msg);

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

        instance.report(&description).await?;

        Ok(())
    }
}
