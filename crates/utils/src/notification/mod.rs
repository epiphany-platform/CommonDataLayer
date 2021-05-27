use crate::notification::full_notification_sender::{
    FullNotificationSender, FullNotificationSenderBase,
};
use cdl_dto::ingestion::OwnMessage;
use communication_utils::publisher::CommonPublisher;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::marker::PhantomData;

pub mod full_notification_sender;

/// This is convenience trait for types that do not implement Serialize (looking at you `tonic`),
/// but we'd prefer to avoid unnecessary cloning of their instances when using NotificationPublisher::Disabled.
pub trait IntoSerialize<S: Serialize> {
    fn into_serialize(self) -> S;
}

impl<S> IntoSerialize<S> for S
where
    S: Serialize,
{
    fn into_serialize(self) -> S {
        self
    }
}

#[derive(Clone)]
pub enum NotificationPublisher<T, S = T>
where
    T: IntoSerialize<S> + Send + Sync + 'static,
    S: Serialize,
{
    Full(FullNotificationSenderBase<T, S>),
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

impl<T, S> NotificationPublisher<T, S>
where
    T: IntoSerialize<S> + Send + Sync + 'static,
    S: Serialize + Send + Sync + 'static,
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
                _phantom: PhantomData,
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
    pub async fn publisher<T, S, F, Fut>(
        &self,
        publisher: F,
        context: String,
        application: &'static str,
    ) -> anyhow::Result<NotificationPublisher<T, S>>
    where
        T: IntoSerialize<S> + Send + Sync + 'static,
        S: Serialize,
        F: Fn() -> Fut,
        Fut: Future<Output = anyhow::Result<CommonPublisher>>,
    {
        Ok(if self.enabled {
            NotificationPublisher::Full(
                FullNotificationSenderBase::new(
                    publisher().await?,
                    self.destination.clone(),
                    context,
                    application,
                )
                .await,
            )
        } else {
            NotificationPublisher::Disabled
        })
    }
}
