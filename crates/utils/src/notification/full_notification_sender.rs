use crate::notification::{IntoSerialize, NotificationService};
use anyhow::Context;
use communication_utils::publisher::CommonPublisher;
use serde::Serialize;
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::{debug, trace};

#[derive(Clone)]
pub struct FullNotificationSenderBase<T, S>
where
    T: IntoSerialize<S> + Send + Sync + 'static,
    S: Serialize,
{
    pub publisher: CommonPublisher,
    pub destination: Arc<String>,
    pub context: Arc<String>,
    pub application: &'static str,
    _phantom: PhantomData<(T, S)>,
}

pub struct FullNotificationSender<T, S>
where
    T: IntoSerialize<S> + Send + Sync + 'static,
    S: Serialize,
{
    pub producer: CommonPublisher,
    pub destination: Arc<String>,
    pub context: Arc<String>,
    pub msg: T,
    pub application: &'static str,
    pub _phantom: PhantomData<S>,
}

#[derive(Serialize)]
struct NotificationBody<'a, T>
where
    T: Serialize + Send + Sync + 'static,
{
    application: &'static str,
    context: &'a str,
    description: &'a str,
    #[serde(flatten)]
    msg: T,
}

impl<T, S> FullNotificationSenderBase<T, S>
where
    T: IntoSerialize<S> + Send + Sync + 'static,
    S: Serialize,
{
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
            _phantom: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<T, S> NotificationService for FullNotificationSender<T, S>
where
    T: IntoSerialize<S> + Send + Sync + 'static,
    S: Serialize + Send + Sync + 'static,
{
    async fn notify(self: Box<Self>, description: &str) -> anyhow::Result<()> {
        let serialize = self.msg.into_serialize();

        trace!(
            "Notification `{}` - `{}`",
            serde_json::to_string(&serialize)
                .unwrap_or_else(|err| format!("failed to serialize json {}", err)),
            description
        );

        let payload = NotificationBody {
            application: self.application,
            context: self.context.as_str(),
            description,
            msg: serialize,
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
