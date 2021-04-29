use crate::output::OutputPlugin;
use crate::{communication::MessageRouter, input::Error};
use async_trait::async_trait;
use futures::future::try_join_all;
use std::sync::Arc;
use tracing::{error, trace};
use utils::communication::get_order_group_id;
use utils::communication::{
    message::CommunicationMessage, parallel_consumer::ParallelConsumerHandler,
};
use utils::communication::{parallel_consumer::ParallelCommonConsumer, Result};
use utils::metrics::{self, counter};
use utils::{message_types::BorrowedInsertMessage, parallel_task_queue::ParallelTaskQueue};

pub struct Service<P: OutputPlugin> {
    consumers: Vec<ParallelCommonConsumer>,
    handler: ServiceHandler<P>,
}

struct ServiceHandler<P: OutputPlugin> {
    message_router: MessageRouter<P>,
    task_queue: Arc<ParallelTaskQueue>,
}

impl<P: OutputPlugin> Clone for ServiceHandler<P> {
    fn clone(&self) -> Self {
        Self {
            message_router: self.message_router.clone(),
            task_queue: self.task_queue.clone(),
        }
    }
}

#[async_trait]
impl<P> ParallelConsumerHandler for ServiceHandler<P>
where
    P: OutputPlugin,
{
    #[tracing::instrument(skip(self, msg))]
    async fn handle<'a>(&'a self, msg: &'a dyn CommunicationMessage) -> anyhow::Result<()> {
        let order_group_id = get_order_group_id(msg);
        let _guard = order_group_id
            .map(move |x| async move { self.task_queue.acquire_permit(x.to_string()).await });

        counter!("cdl.command-service.input-request", 1);

        let generic_message = Self::build_message(msg)?;

        trace!("Received message {:?}", generic_message);

        self.message_router.handle_message(generic_message).await?;

        Ok(())
    }
}

impl<P: OutputPlugin> ServiceHandler<P> {
    fn build_message(
        message: &'_ dyn CommunicationMessage,
    ) -> Result<BorrowedInsertMessage<'_>, Error> {
        let json = message.payload().map_err(Error::MissingPayload)?;
        let event: BorrowedInsertMessage =
            serde_json::from_str(json).map_err(Error::PayloadDeserializationFailed)?;

        Ok(event)
    }
}

impl<P: OutputPlugin> Service<P> {
    pub async fn new(
        consumers: Vec<ParallelCommonConsumer>,
        message_router: MessageRouter<P>,
    ) -> Result<Self, Error> {
        Ok(Self {
            consumers,
            handler: ServiceHandler {
                message_router,
                task_queue: Arc::new(ParallelTaskQueue::default()),
            },
        })
    }

    pub async fn listen(self) -> Result<(), Error> {
        trace!("Number of consumers: {}", self.consumers.len());

        let mut futures = vec![];
        for consumer in self.consumers {
            futures.push(consumer.par_run(self.handler.clone()));
        }

        if let Err(err) = try_join_all(futures).await {
            error!("Failed to handle message: {}", err);
        }

        trace!("Stream closed");

        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        Ok(())
    }
}
