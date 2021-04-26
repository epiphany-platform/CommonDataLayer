use crate::communication::publisher::CommonPublisher;
use crate::message_types::OwnedInsertMessage;
use crate::notification::NotificationService;
use anyhow::Context;
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, trace};
use uuid::Uuid;

#[derive(Clone)]
pub struct FullNotificationSenderBase {
    pub publisher: CommonPublisher,
    pub destination: Arc<String>,
    pub context: Arc<String>,
    pub application: &'static str,
}

pub struct FullNotificationSender {
    pub producer: CommonPublisher,
    pub destination: Arc<String>,
    pub context: Arc<String>,
    pub msg: OwnedInsertMessage,
    pub application: &'static str,
}

#[derive(Serialize)]
struct NotificationBody<'a> {
    application: &'static str,
    context: &'a str,
    description: &'a str,
    schema_id: Uuid,
    object_id: Uuid,
    payload: Value,
}

impl FullNotificationSenderBase {
    pub async fn new(
        publisher: CommonPublisher,
        destination: String,
        context: String,
        application: &'static str,
    ) -> Self {
        debug!(
            "Initialized Notification service with sink at `{}`",
            destination
        );

        Self {
            publisher,
            destination: Arc::new(destination),
            context: Arc::new(context),
            application,
        }
    }
}

#[async_trait::async_trait]
impl NotificationService for FullNotificationSender {
    async fn notify(self: Box<Self>, description: &str) -> anyhow::Result<()> {
        trace!(
            "Notification for id `{}` - `{}`",
            self.msg.object_id,
            description
        );

        let payload = NotificationBody {
            application: self.application,
            context: self.context.as_str(),
            description,
            schema_id: self.msg.schema_id,
            object_id: self.msg.object_id,
            payload: self.msg.data,
        };

        self.producer
            .publish_message(
                self.destination.as_str(),
                &format!("{}.status", self.application),
                serde_json::to_vec(&payload).context("Failed to serialize json")?,
            )
            .await
            .context("Failed to send notification")
    }
}
