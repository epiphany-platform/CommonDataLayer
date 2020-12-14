use anyhow::Context;
use async_trait::async_trait;
use lapin::{message::Delivery, options::BasicAckOptions, Channel};
use rdkafka::{
    consumer::CommitMode,
    consumer::{DefaultConsumerContext, StreamConsumer},
    message::BorrowedMessage,
    Message,
};
use std::sync::Arc;

use super::Result;

#[async_trait]
pub trait CommunicationMessage: Send + Sync {
    fn payload(&self) -> Result<&str>;
    fn key(&self) -> Result<&str>;
    fn timestamp(&self) -> Result<i64>;
    async fn ack(&self) -> Result<()>;
}

pub struct KafkaCommunicationMessage<'a> {
    pub(super) message: BorrowedMessage<'a>,
    pub(super) consumer: Arc<StreamConsumer<DefaultConsumerContext>>,
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

    fn timestamp(&self) -> Result<i64> {
        self.message
            .timestamp()
            .to_millis()
            .ok_or_else(|| anyhow::anyhow!("Message has no timestamp").into())
    }

    async fn ack(&self) -> Result<()> {
        rdkafka::consumer::Consumer::commit_message(
            self.consumer.as_ref(),
            &self.message,
            CommitMode::Async,
        )?;
        Ok(())
    }
}

pub struct RabbitCommunicationMessage {
    pub(super) channel: Channel,
    pub(super) delivery: Delivery,
}

#[async_trait]
impl CommunicationMessage for RabbitCommunicationMessage {
    fn key(&self) -> Result<&str> {
        let key = self.delivery.routing_key.as_str();
        Ok(key)
    }

    fn payload(&self) -> Result<&str> {
        Ok(std::str::from_utf8(&self.delivery.data).context("Payload was not valid UTF-8")?)
    }

    fn timestamp(&self) -> Result<i64> {
        self.delivery
            .properties
            .timestamp()
            .map(|t| t as i64)
            .ok_or_else(|| anyhow::anyhow!("Message has no timestamp").into())
    }

    async fn ack(&self) -> Result<()> {
        Ok(self
            .channel
            .basic_ack(self.delivery.delivery_tag, BasicAckOptions::default())
            .await?)
    }
}
