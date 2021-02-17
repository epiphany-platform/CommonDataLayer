use crate::output::OutputPlugin;
use crate::{
    communication::{config::MessageQueueConfig, MessageRouter},
    input::Error,
};
use futures::stream::select_all;
use futures::stream::StreamExt;
use log::{error, trace};
use std::{process, sync::Arc};
use tokio::pin;
use utils::messaging_system::consumer::CommonConsumer;
use utils::messaging_system::message::CommunicationMessage;
use utils::messaging_system::Result;
use utils::metrics::counter;
use utils::task_limiter::TaskLimiter;
use utils::{message_types::BorrowedInsertMessage, parallel_task_queue::ParallelTaskQueue};

pub struct MessageQueueInput<P: OutputPlugin> {
    consumer: Vec<CommonConsumer>,
    message_router: MessageRouter<P>,
    task_limiter: TaskLimiter,
    task_queue: Arc<ParallelTaskQueue>,
}

impl<P: OutputPlugin> MessageQueueInput<P> {
    pub async fn new(
        config: MessageQueueConfig,
        message_router: MessageRouter<P>,
    ) -> Result<Self, Error> {
        let mut consumers = Vec::new();
        for ordered in config.ordered_configs() {
            let consumer = CommonConsumer::new(ordered)
                .await
                .map_err(Error::ConsumerCreationFailed)?;
            consumers.push(consumer);
        }

        for unordered in config.unordered_configs() {
            let consumer = CommonConsumer::new(unordered)
                .await
                .map_err(Error::ConsumerCreationFailed)?;
            consumers.push(consumer);
        }

        Ok(Self {
            consumer: consumers,
            message_router,
            task_limiter: TaskLimiter::new(config.task_limit()),
            task_queue: Arc::new(ParallelTaskQueue::default()),
        })
    }

    async fn handle_message(
        router: MessageRouter<P>,
        message: Result<Box<dyn CommunicationMessage>>,
        task_queue: Arc<ParallelTaskQueue>,
    ) -> Result<(), Error> {
        counter!("cdl.command-service.input-request", 1);
        let message = message.map_err(Error::FailedReadingMessage)?;

        let generic_message = Self::build_message(message.as_ref())?;

        trace!("Received message {:?}", generic_message);

        let _guard = generic_message
            .order_group_id
            .map(move |x| async move { task_queue.acquire_permit(x.to_string()).await });

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
        let mut streams = Vec::new();
        trace!("Number of consumers: {}", self.consumer.len());
        for consumer in self.consumer {
            let consumer = consumer.leak(); // Limits ability to change queues CS is listening on without restarting whole service
            let stream = consumer.consume().await;
            streams.push(stream);
        }

        trace!("Beginning to listen on {} stream(s)", streams.len());

        let message_stream = select_all(streams.into_iter().map(Box::pin));

        pin!(message_stream);

        trace!("Listen on message stream");
        while let Some(message) = message_stream.next().await {
            let router = self.message_router.clone();

            let task_queue = self.task_queue.clone();
            self.task_limiter
                .run(move || async move {
                    if let Err(err) =
                        MessageQueueInput::handle_message(router, message, task_queue).await
                    {
                        error!("Failed to handle message: {}", err);
                        process::abort();
                    }
                })
                .await;
        }
        trace!("Stream closed");

        tokio::time::delay_for(tokio::time::Duration::from_secs(3)).await;

        Ok(())
    }
}
