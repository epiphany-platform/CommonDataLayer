#![allow(unused_imports)]
use anyhow::Context;
#[cfg(feature = "amqp")]
use lapin::message::Delivery;
#[cfg(feature = "kafka")]
use rdkafka::{message::BorrowedMessage, Message};

use super::Result;

pub trait CommunicationMessage: Send + Sync {
    fn payload(&self) -> Result<&str>;
    fn key(&self) -> Result<&str>;
}

#[cfg(feature = "kafka")]
pub struct KafkaCommunicationMessage<'a> {
    pub(super) message: BorrowedMessage<'a>,
}
#[cfg(feature = "kafka")]
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
}

#[cfg(feature = "amqp")]
pub struct AmqpCommunicationMessage {
    pub(super) delivery: Delivery,
}
#[cfg(feature = "amqp")]
impl CommunicationMessage for AmqpCommunicationMessage {
    fn key(&self) -> Result<&str> {
        let key = self.delivery.routing_key.as_str();
        Ok(key)
    }
    fn payload(&self) -> Result<&str> {
        Ok(std::str::from_utf8(&self.delivery.data).context("Payload was not valid UTF-8")?)
    }
}

#[cfg(feature = "grpc")]
impl CommunicationMessage for rpc::generic::Message {
    fn payload(&self) -> Result<&str> {
        Ok(std::str::from_utf8(&self.payload)?)
    }

    fn key(&self) -> Result<&str> {
        Ok(&self.key)
    }
}
