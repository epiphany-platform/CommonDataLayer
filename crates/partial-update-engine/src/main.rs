use std::collections::HashSet;

use anyhow::Context;
use rdkafka::{
    consumer::{DefaultConsumerContext, StreamConsumer},
    error::KafkaError,
    ClientConfig,
};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use tokio::stream::StreamExt;
use tracing::debug;
use utils::metrics::{self};
use uuid::Uuid;

#[derive(StructOpt, Deserialize, Debug, Serialize)]
struct Config {
    /// Address of Kafka brokers
    #[structopt(long, env)]
    pub brokers: String,
    /// Group ID of the consumer
    #[structopt(long, env)]
    pub group_id: String,
    /// Kafka topic
    #[structopt(long, env)]
    pub notification_topic: String,
    /// Address of schema registry gRPC API
    #[structopt(long, env)]
    pub schema_registry_addr: String,
    /// How many entries the cache can hold
    #[structopt(long, env)]
    pub cache_capacity: usize,
    /// Port to listen on for Prometheus requests
    #[structopt(default_value = metrics::DEFAULT_PORT, env)]
    pub metrics_port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();
    utils::tracing::init();

    let config: Config = Config::from_args();

    debug!("Environment {:?}", config);

    metrics::serve(config.metrics_port);

    let consumer: StreamConsumer<DefaultConsumerContext> = ClientConfig::new()
        .set("group.id", &config.group_id)
        .set("bootstrap.servers", &config.brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set("enable.auto.offset.store", "false")
        .set("auto.offset.reset", "earliest")
        .create()
        .context("Consumer creation failed")?;
    let topics = [config.notification_topic.as_str()];

    rdkafka::consumer::Consumer::subscribe(&consumer, &topics)
        .context("Can't subscribe to specified topics")?;

    let mut message_stream = consumer.start();

    let mut changes: HashSet<(Uuid, Uuid)> = HashSet::new();
    loop {
        match message_stream.try_next().await {
            Ok(opt_message) => match opt_message {
                Some(message) => {
                    // changes.insert()
                    // TODO: process message
                }
                None => {
                    // TODO: send  requests to object builder, acks
                    break;
                }
            },
            Err(err) => match err {
                KafkaError::PartitionEOF(_) => {
                    // TODO: send  requests to object builder, acks
                }
                err => {
                    panic!("Unknown kafka error {:?}", err)
                }
            },
        }
        // let mut partition_offsets = TopicPartitionList::new();
        //         partition_offsets.add_partition_offset(
        //             &self.topic,
        //             self.partition,
        //             Offset::Offset(offset),
        //         );
        //         rdkafka::consumer::Consumer::store_offsets(&consumer, &partition_offsets).unwrap();
    }

    tokio::time::delay_for(tokio::time::Duration::from_secs(3)).await;

    Ok(())
}
