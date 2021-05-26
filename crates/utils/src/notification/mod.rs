use crate::notification::full_notification_sender::{
    FullNotificationSender, FullNotificationSenderBase,
};
use cdl_dto::ingestion::OwnMessage;
use communication_utils::publisher::CommonPublisher;
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct NotificationSettings {
    /// Kafka topic, AMQP queue or GRPC url
    #[serde(default)]
    pub destination: String,
    #[serde(default)]
    pub enabled: bool,
}

impl NotificationSettings {
    pub async fn publisher<T: Serialize + Send + Sync + 'static>(
        &self,
        publisher: CommonPublisher, // FIXME
        context: String,
        application: &'static str,
    ) -> NotificationPublisher<T> {
        if self.enabled {
            NotificationPublisher::Full(
                FullNotificationSenderBase::new(
                    publisher,
                    self.destination.clone(),
                    context,
                    application,
                )
                .await,
            )
        } else {
            NotificationPublisher::Disabled
        }
    }
}
