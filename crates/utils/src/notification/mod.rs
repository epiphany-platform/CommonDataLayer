use crate::message_types::OwnMessage;
use crate::notification::full_notification_sender::{
    FullNotificationSender, FullNotificationSenderBase,
};
use serde::Serialize;

pub mod full_notification_sender;

#[derive(Clone)]
pub enum NotificationPublisher<T>
where
    T: Serialize + Send + Sync + 'static,
{
    Full(FullNotificationSenderBase<T>),
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

impl<T> NotificationPublisher<T>
where
    T: Serialize + Send + Sync + 'static,
{
    pub fn with_message_body<U>(self, msg: &U) -> Box<dyn NotificationService>
    where
        U: OwnMessage<Owned = T>,
    {
        match self {
            NotificationPublisher::Full(config) => Box::new(FullNotificationSender {
                application: config.application,
                producer: config.publisher,
                destination: config.destination,
                context: config.context,
                msg: msg.to_owned_message(),
            }),
            NotificationPublisher::Disabled => Box::new(()),
        }
    }
}
