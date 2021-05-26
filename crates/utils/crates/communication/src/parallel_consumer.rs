#![allow(unused_imports, unused_variables)]
use anyhow::Context;
use async_trait::async_trait;
use futures_util::TryStreamExt;
#[cfg(feature = "amqp")]
pub use lapin::options::BasicConsumeOptions;
#[cfg(feature = "amqp")]
use lapin::types::FieldTable;
#[cfg(feature = "kafka")]
use rdkafka::{
    consumer::{DefaultConsumerContext, StreamConsumer},
    ClientConfig,
};
#[cfg(feature = "grpc")]
use rpc::generic as proto;
#[cfg(feature = "grpc")]
use rpc::generic::generic_rpc_server::GenericRpc;
#[cfg(feature = "grpc")]
use rpc::generic::generic_rpc_server::GenericRpcServer;
use std::{net::SocketAddrV4, sync::Arc};
use task_utils::task_limiter::TaskLimiter;
#[cfg(feature = "amqp")]
use tokio_amqp::LapinTokioExt;
use tracing_futures::Instrument;

#[cfg(feature = "amqp")]
use super::message::AmqpCommunicationMessage;
#[cfg(feature = "kafka")]
use super::{kafka_ack_queue::KafkaAckQueue, message::KafkaCommunicationMessage};
use super::{message::CommunicationMessage, Result};

#[async_trait]
pub trait ParallelConsumerHandler: Send + Sync + 'static {
    async fn handle<'a>(&'a self, msg: &'a dyn CommunicationMessage) -> anyhow::Result<()>;
}

#[cfg(feature = "grpc")]
struct GenericRpcImpl<T> {
    handler: Arc<T>,
}

#[cfg(feature = "grpc")]
#[tonic::async_trait]
impl<T> GenericRpc for GenericRpcImpl<T>
where
    T: ParallelConsumerHandler,
{
    #[tracing::instrument(skip(self))]
    async fn handle(
        &self,
        request: tonic::Request<proto::Message>,
    ) -> Result<tonic::Response<proto::Empty>, tonic::Status> {
        let msg = request.into_inner();

        match self.handler.handle(&msg).await {
            Ok(_) => Ok(tonic::Response::new(proto::Empty {})),
            Err(err) => Err(tonic::Status::internal(err.to_string())),
        }
    }
}

pub enum ParallelCommonConsumerConfig<'a> {
    #[cfg(feature = "kafka")]
    Kafka {
        brokers: &'a str,
        group_id: &'a str,
        topic: &'a str,
        task_limiter: TaskLimiter,
    },
    #[cfg(feature = "amqp")]
    Amqp {
        connection_string: &'a str,
        consumer_tag: &'a str,
        queue_name: &'a str,
        options: Option<BasicConsumeOptions>,
        task_limiter: TaskLimiter,
    },
    #[cfg(feature = "grpc")]
    Grpc { addr: SocketAddrV4 },
}
pub enum ParallelCommonConsumer {
    #[cfg(feature = "kafka")]
    Kafka {
        consumer: StreamConsumer<DefaultConsumerContext>,
        ack_queue: KafkaAckQueue,
        task_limiter: TaskLimiter,
    },
    #[cfg(feature = "amqp")]
    Amqp {
        consumer: lapin::Consumer,
        task_limiter: TaskLimiter,
    },
    #[cfg(feature = "grpc")]
    Grpc { addr: SocketAddrV4 },
}
impl ParallelCommonConsumer {
    pub async fn new(config: ParallelCommonConsumerConfig<'_>) -> Result<Self> {
        match config {
            #[cfg(feature = "kafka")]
            ParallelCommonConsumerConfig::Kafka {
                group_id,
                brokers,
                topic,
                task_limiter,
            } => Self::new_kafka(group_id, brokers, &[topic], task_limiter).await,
            #[cfg(feature = "amqp")]
            ParallelCommonConsumerConfig::Amqp {
                connection_string,
                consumer_tag,
                queue_name,
                options,
                task_limiter,
            } => {
                Self::new_amqp(
                    connection_string,
                    consumer_tag,
                    queue_name,
                    options,
                    task_limiter,
                )
                .await
            }
            #[cfg(feature = "grpc")]
            ParallelCommonConsumerConfig::Grpc { addr } => Ok(Self::new_grpc(addr)),
        }
    }

    #[cfg(feature = "grpc")]
    #[tracing::instrument]
    fn new_grpc(addr: SocketAddrV4) -> Self {
        tracing::debug!("Creating GRPC parallel consumer");
        Self::Grpc { addr }
    }

    #[cfg(feature = "kafka")]
    #[tracing::instrument]
    async fn new_kafka(
        group_id: &str,
        brokers: &str,
        topics: &[&str],
        task_limiter: TaskLimiter,
    ) -> Result<Self> {
        tracing::debug!("Creating kafka parallel consumer");
        let consumer: StreamConsumer<DefaultConsumerContext> = ClientConfig::new()
            .set("group.id", group_id)
            .set("bootstrap.servers", brokers)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .set("enable.auto.offset.store", "false")
            .set("auto.offset.reset", "earliest")
            .create()
            .context("Consumer creation failed")?;

        eprintln!("{:?}", topics);

        rdkafka::consumer::Consumer::subscribe(&consumer, topics)
            .context("Can't subscribe to specified topics")?;

        Ok(Self::Kafka {
            consumer,
            task_limiter,
            ack_queue: Default::default(),
        })
    }

    #[cfg(feature = "amqp")]
    #[tracing::instrument]
    async fn new_amqp(
        connection_string: &str,
        consumer_tag: &str,
        queue_name: &str,
        consume_options: Option<BasicConsumeOptions>,
        task_limiter: TaskLimiter,
    ) -> Result<Self> {
        tracing::debug!("Creating AMQP parallel consumer");
        let consume_options = consume_options.unwrap_or_default();
        let connection = lapin::Connection::connect(
            connection_string,
            lapin::ConnectionProperties::default().with_tokio(),
        )
        .await?;
        let channel = connection.create_channel().await?;
        let consumer = channel
            .basic_consume(
                queue_name,
                consumer_tag,
                consume_options,
                FieldTable::default(),
            )
            .await?;
        Ok(Self::Amqp {
            consumer,
            task_limiter,
        })
    }

    /// Process messages in parallel
    /// # Memory safety
    /// This method leaks kafka consumer
    /// # Error handling
    /// ## Kafka & AMQP
    /// Program exits on first unhandled message. I may cause crash-loop.
    /// ## GRPC
    /// Program returns 500 code and tries to handle further messages.
    pub async fn par_run(self, handler: impl ParallelConsumerHandler) -> Result<()> {
        let handler = Arc::new(handler);
        match self {
            #[cfg(feature = "kafka")]
            Self::Kafka {
                consumer,
                ack_queue,
                task_limiter,
            } => {
                let consumer = Box::leak(Box::new(Arc::new(consumer)));
                let ack_queue = Arc::new(ack_queue);
                let mut message_stream = consumer.stream();
                while let Some(message) = message_stream.try_next().await? {
                    ack_queue.add(&message);
                    let ack_queue = ack_queue.clone();
                    let handler = handler.clone();
                    let consumer = consumer.clone();
                    let span = tracing::info_span!("consume_message");
                    task_limiter
                        .run(move || {
                            async move {
                                tracing_utils::kafka::set_parent_span(&message);
                                let message = KafkaCommunicationMessage { message };

                                match handler.handle(&message).await {
                                    Ok(_) => {
                                        ack_queue.ack(&message.message, consumer.as_ref());
                                    }
                                    Err(e) => {
                                        tracing::error!(?e, "Couldn't process message");
                                    }
                                }
                            }
                            .instrument(span)
                        })
                        .await;
                }
            }
            #[cfg(feature = "amqp")]
            Self::Amqp {
                mut consumer,
                task_limiter,
            } => {
                while let Some((channel, delivery)) = consumer.try_next().await? {
                    let handler = handler.clone();
                    task_limiter
                        .run(move || async move {
                            let message = AmqpCommunicationMessage { delivery };
                            match handler.handle(&message).await {
                                Ok(_) => {
                                    if let Err(e) = channel
                                        .basic_ack(
                                            message.delivery.delivery_tag,
                                            Default::default(),
                                        )
                                        .await
                                    {
                                        tracing::error!(?e, "Couldn't ack message");
                                    }
                                }
                                Err(e) => {
                                    tracing::error!(?e, "Couldn't process message");
                                }
                            }
                        })
                        .await;
                }
            }
            #[cfg(feature = "grpc")]
            Self::Grpc { addr } => {
                tonic::transport::Server::builder()
                    .trace_fn(tracing_utils::grpc::trace_fn)
                    .add_service(GenericRpcServer::new(GenericRpcImpl { handler }))
                    .serve(addr.into())
                    .await?;
            }
        }
        Ok(())
    }
}
