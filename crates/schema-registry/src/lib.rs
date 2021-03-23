#![feature(drain_filter)]

pub mod cache;
pub mod db;
pub mod error;
pub mod rpc;
pub mod types;
pub mod utils;

pub enum CommunicationMethodConfig {
    Kafka(KafkaConfig),
    Amqp(AmqpConfig),
    Grpc,
}

#[derive(Clone, Debug)]
pub struct KafkaConfig {
    pub brokers: String,
    pub group_id: String,
}

#[derive(Clone, Debug)]
pub struct AmqpConfig {
    pub connection_string: String,
    pub consumer_tag: String,
}
