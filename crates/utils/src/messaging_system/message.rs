use anyhow::Context;
use async_trait::async_trait;
use lapin::{message::Delivery, options::BasicAckOptions, Channel};
use rdkafka::{
    consumer::{DefaultConsumerContext, StreamConsumer},
    message::BorrowedMessage,
    Message,
};
use std::sync::Arc;

use super::{kafka_ack_queue::KafkaAckQueue, Result};

#[async_trait]
pub trait CommunicationMessage: Send + Sync {
    fn payload(&self) -> Result<&str>;
    fn key(&self) -> Result<&str>;
    async fn ack(&self) -> Result<()>;
}

pub struct KafkaCommunicationMessage<'a> {
    pub(super) message: BorrowedMessage<'a>,
    pub(super) consumer: Arc<StreamConsumer<DefaultConsumerContext>>,
    pub(super) ack_queue: Arc<KafkaAckQueue>,
}
#[async_trait]
impl<'a> CommunicationMessage for KafkaCommunicationMessage<'a> {
    fn key(&self) -> Result<&str> {
        let key = self
            .message
            .key()
            .ok_or_else(|| anyhow::anyhow!("Message has no key"))?;
        Ok(std::str::from_utf8(key)?)
    }
    fn payload(&self) -> Result<&str> {
        Ok(self
            .message
            .payload_view::<str>()
            .ok_or_else(|| anyhow::anyhow!("Message has no payload"))??)
    }
    async fn ack(&self) -> Result<()> {
        self.ack_queue.ack(&self.message, self.consumer.as_ref());
        Ok(())
    }
}

pub struct AmqpCommunicationMessage {
    pub(super) channel: Channel,
    pub(super) delivery: Delivery,
}
#[async_trait]
impl CommunicationMessage for AmqpCommunicationMessage {
    fn key(&self) -> Result<&str> {
        let key = self.delivery.routing_key.as_str();
        Ok(key)
    }
    fn payload(&self) -> Result<&str> {
        Ok(std::str::from_utf8(&self.delivery.data).context("Payload was not valid UTF-8")?)
    }
    async fn ack(&self) -> Result<()> {
        Ok(self
            .channel
            .basic_ack(self.delivery.delivery_tag, BasicAckOptions::default())
            .await?)
    }
}
