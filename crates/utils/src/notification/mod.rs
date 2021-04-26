use crate::message_types::BorrowedInsertMessage;
use crate::notification::full_notification_sender::{
    FullNotificationSender, FullNotificationSenderBase,
};
pub use config::NotificationServiceConfig;

mod config;
pub mod full_notification_sender;

#[derive(Clone)]
pub enum NotificationSender {
    Full(FullNotificationSenderBase),
    Disabled,
}

#[async_trait::async_trait]
pub trait NotificationService: Send + Sync + 'static {
    async fn notify(self: Box<Self>, description: &str) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
impl NotificationService for () {
    async fn notify(self: Box<Self>, _: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

impl NotificationSender {
    pub fn with_message_body(self, msg: &BorrowedInsertMessage) -> Box<dyn NotificationService> {
        match self {
            NotificationSender::Full(config) => Box::new(FullNotificationSender {
                application: config.application,
                producer: config.publisher,
                destination: config.destination,
                context: config.context,
                msg: msg.to_owned(),
            }),
            NotificationSender::Disabled => Box::new(()),
        }
    }
}
