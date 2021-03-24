use crate::communication::resolution::Resolution;
use crate::output::OutputPlugin;
use crate::report::{Error, ReportSender};
use std::sync::Arc;
use tracing::trace;
use utils::message_types::BorrowedInsertMessage;
use utils::metrics::*;

pub mod config;
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

        trace!("Finished processing a message with resolution `{}`", status);

        match status {
            Resolution::StorageLayerFailure { .. } => {
                counter!("cdl.command-service.post-process.storage-failure", 1);
            }
            Resolution::CommandServiceFailure => {
                counter!("cdl.command-service.post-process.command-failure", 1);
            }
            Resolution::UserFailure { .. } => {
                counter!("cdl.command-service.post-process.user-failure", 1);
            }
            Resolution::Success => {
                counter!("cdl.command-service.post-process.success", 1);
            }
        }

        instance.report(&status.to_string()).await?;

        Ok(())
    }
}
