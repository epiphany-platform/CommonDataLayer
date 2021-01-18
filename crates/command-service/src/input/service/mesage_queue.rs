use crate::output::OutputPlugin;
use crate::{
    communication::MessageRouter,
    input::{Error, MessageQueueConfig},
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
use utils::{
    message_types::BorrowedInsertMessage, messaging_system::consumer::BasicConsumeOptions,
    parallel_task_queue::ParallelTaskQueue,
};

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
        let ordered_options = Some(BasicConsumeOptions {
            exclusive: true,
            ..Default::default()
        });
        let unordered_options = Some(BasicConsumeOptions {
            exclusive: false,
            ..Default::default()
        });
        for queue_name in config.ordered_queue_names {
            let consumer = CommonConsumer::new_rabbit(
                &config.connection_string,
                &config.consumer_tag,
                &queue_name,
                ordered_options,
            )
            .await
            .map_err(Error::ConsumerCreationFailed)?;
            consumers.push(consumer);
        }
        for queue_name in config.unordered_queue_names {
            let consumer = CommonConsumer::new_rabbit(
                &config.connection_string,
                &config.consumer_tag,
                &queue_name,
                unordered_options,
            )
            .await
            .map_err(Error::ConsumerCreationFailed)?;
            consumers.push(consumer);
        }

        Ok(Self {
            consumer: consumers,
            message_router,
            task_limiter: TaskLimiter::new(config.task_limit),
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
            .map(async move |x| task_queue.acquire_permit(x.to_string()).await);

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
        for consumer in self.consumer {
            let consumer = consumer.leak(); // Limits ability to change queues CS is listening on without restarting whole service
            let stream = consumer.consume().await;
            streams.push(stream);
        }
        let message_stream = select_all(streams.into_iter().map(Box::pin));

        pin!(message_stream);

        while let Some(message) = message_stream.next().await {
            let router = self.message_router.clone();

            let task_queue = self.task_queue.clone();
            self.task_limiter
                .run(async move || {
                    if let Err(err) =
                        MessageQueueInput::handle_message(router, message, task_queue).await
                    {
                        error!("Failed to handle message: {}", err);
                        process::abort();
                    }
                })
                .await;
        }

        Ok(())
    }
}
