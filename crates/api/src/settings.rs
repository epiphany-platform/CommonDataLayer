use utils::settings::*;
use serde::Deserialize;
use utils::communication::publisher::CommonPublisher;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub communication_method: CommunicationMethod,
    pub input_port: u16,

    pub kafka: Option<ConsumerKafkaSettings>,
    pub amqp: Option<AmqpSettings>,

    pub services: ServicesSettings,

    pub notification_consumer: Option<NotificationConsumerSettings>,
    pub insert_destination: String,
}

#[derive(Debug, Deserialize)]
pub struct ServicesSettings {
    pub schema_registry_url: String,
    pub edge_registry_url: String,
    pub on_demand_materializer_url: String,
    pub query_router_url: String,
}

#[derive(Deserialize, Debug)]
pub struct NotificationConsumerSettings {
    pub source: String,
}

impl Settings {
    pub async fn publisher(&self) -> anyhow::Result<CommonPublisher> {
        match (&self.kafka, &self.amqp, &self.communication_method) {
            (Some(kafka), _, CommunicationMethod::Kafka) => {
                Ok(CommonPublisher::new_kafka(&kafka.brokers).await?)
            },
            (_, Some(amqp), CommunicationMethod::Amqp) => {
                Ok(CommonPublisher::new_amqp(&amqp.exchange_url).await?)
            },
            (_, _, CommunicationMethod::GRpc) => {
                Ok(CommonPublisher::new_grpc().await?)
            },
            _ => todo!(),
        }
    }
}
