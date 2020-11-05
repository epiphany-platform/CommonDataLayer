use std::sync::Arc;

use futures_util::stream::StreamExt;
use log::{log_enabled, trace, Level};
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::{Consumer, DefaultConsumerContext, StreamConsumer};
use rdkafka::error::KafkaError;
use rdkafka::message::{Headers, OwnedMessage};
use rdkafka::{ClientConfig, Message};
use uuid::Uuid;

use crate::communication::{GenericMessage, MessageRouter};
use crate::input::{Error, KafkaInputConfig};
use crate::output::OutputPlugin;
use utils::metrics::counter;

pub struct KafkaInput<P: OutputPlugin> {
    consumer: Arc<StreamConsumer<DefaultConsumerContext>>,
    message_router: Arc<MessageRouter<P>>,
    // task_limiter: TaskLimiter,
}

impl<P: OutputPlugin> KafkaInput<P> {
    pub fn new(config: KafkaInputConfig, message_router: MessageRouter<P>) -> Result<Self, Error> {
        let consumer: StreamConsumer<DefaultConsumerContext> = ClientConfig::default()
            .set("group.id", &config.group_id)
            .set("bootstrap.servers", &config.brokers)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .set_log_level(RDKafkaLogLevel::Debug)
            .create_with_context(DefaultConsumerContext)
            .map_err(Error::ConsumerCreationFailed)?;

        consumer
            .subscribe(&[&config.topic])
            .map_err(Error::FailedToSubscribe)?;

        Ok(Self {
            consumer: Arc::new(consumer),
            message_router: Arc::new(message_router),
        })
    }

    async fn handle_message(
        router: Arc<MessageRouter<P>>,
        message: Result<OwnedMessage, KafkaError>,
    ) -> Result<(), Error> {
        counter!("cdl.command-service.input-request", 1);
        let message = message.map_err(Error::FailedReadingMessage)?;

        let generic_message = Self::build_message(&message)?;

        router
            .handle_message(generic_message)
            .await
            .map_err(Error::CommunicationError)?;

        Ok(())
    }

    fn build_message(message: &OwnedMessage) -> Result<GenericMessage, Error> {
        let key = std::str::from_utf8(message.key().ok_or(Error::MissingKey)?)
            .map_err(Error::UnableToParseUTF8)?;
        let object_id = Uuid::parse_str(key).map_err(Error::KeyNotValidUuid)?;
        let schema_id = Self::get_schema_id(message)?;
        let payload = message.payload().ok_or(Error::MissingPayload)?.to_vec();
        let ts = message.timestamp();

        Ok(GenericMessage {
            object_id,
            schema_id,
            timestamp: ts.to_millis().ok_or(Error::TimestampUnavailable)?,
            payload,
        })
    }

    fn get_schema_id(message: &OwnedMessage) -> Result<Uuid, Error> {
        let schema_id_header = message.headers().and_then(|headers| {
            for index in 0..headers.count() {
                let (name, value) = headers.get(index).unwrap();
                if name == "SCHEMA_ID" {
                    return Some(value);
                }
            }

            None
        });

        if let Some(header) = schema_id_header {
            Uuid::parse_str(&String::from_utf8_lossy(header).to_string())
                .map_err(|_err| Error::InvalidSchemaIdHeader)
        } else {
            Err(Error::MissingSchemaIdHeader)
        }
    }

    pub async fn listen(&self) -> Result<(), Error> {
        let mut message_stream = self.consumer.start();

        while let Some(message) = message_stream.next().await {
            let router = self.message_router.clone();
            let message = message.map(|msg| msg.detach());

            if log_enabled!(Level::Trace) {
                if let Ok(ref message) = message {
                    let payload = message.payload().map(String::from_utf8_lossy);
                    let key = message.key().map(String::from_utf8_lossy);
                    trace!(
                        r#"Received Message {{ payload: {:?}, key: {:?}, topic: "{}", timestamp: {:?}, partition: {}, offset: {}, headers: {:?} }}"#,
                        payload,
                        key,
                        message.topic(),
                        message.timestamp(),
                        message.partition(),
                        message.offset(),
                        message.headers()
                    );
                }
            }

            Self::handle_message(router, message).await.unwrap()
        }

        Ok(())
    }
}
