use crate::output::OutputPlugin;
use crate::{
    communication::MessageRouter,
    input::{Error, KafkaConfig},
};
use log::{error, trace};
use std::process;
use tokio::pin;
use tokio::stream::StreamExt;
use utils::message_types::BorrowedInsertMessage;
use utils::messaging_system::consumer::CommonConsumer;
use utils::messaging_system::message::CommunicationMessage;
use utils::messaging_system::Result;
use utils::metrics::counter;
use utils::task_limiter::TaskLimiter;

pub struct KafkaInput<P: OutputPlugin> {
    consumer: CommonConsumer,
    message_router: MessageRouter<P>,
    task_limiter: TaskLimiter,
}

impl<P: OutputPlugin> KafkaInput<P> {
    pub async fn new(config: KafkaConfig, message_router: MessageRouter<P>) -> Result<Self, Error> {
        let consumer =
            CommonConsumer::new_kafka(&config.group_id, &config.brokers, &[&config.topic])
                .await
                .map_err(Error::ConsumerCreationFailed)?;

        Ok(Self {
            consumer,
            message_router,
            task_limiter: TaskLimiter::new(config.task_limit),
        })
    }

    async fn handle_message(
        router: MessageRouter<P>,
        message: Result<Box<dyn CommunicationMessage>>,
    ) -> Result<(), Error> {
        counter!("cdl.command-service.input-request", 1);
        let message = message.map_err(Error::FailedReadingMessage)?;

        let generic_message = Self::build_message(message.as_ref())?;

        trace!("Received message {:?}", generic_message);

        router
            .handle_message(generic_message)
            .await
            .map_err(Error::CommunicationError)?;

        message.ack().await.map_err(Error::FailedToAcknowledge)?;

        Ok(())
    }

    fn build_message(
        message: &'_ dyn CommunicationMessage,
    ) -> Result<BorrowedInsertMessage<'_>, Error> {
        let json = message.payload().map_err(Error::MissingPayload)?;
        let event: BorrowedInsertMessage =
            serde_json::from_str(json).map_err(Error::PayloadDeserializationFailed)?;

        Ok(event)
    }

    pub async fn listen(self) -> Result<(), Error> {
        let consumer = self.consumer.leak();
        let message_stream = consumer.consume().await;
        pin!(message_stream);

        while let Some(message) = message_stream.next().await {
            let router = self.message_router.clone();

            self.task_limiter
                .run(async move || {
                    if let Err(err) = Self::handle_message(router, message).await {
                        error!("Failed to handle message: {}", err);
                        process::abort();
                    }
                })
                .await;
        }

        Ok(())
    }
}
