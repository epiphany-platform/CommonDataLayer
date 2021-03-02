use std::net::{Ipv4Addr, SocketAddrV4};
use url::Url;

use utils::messaging_system::consumer::{BasicConsumeOptions, CommonConsumerConfig};

pub enum CommunicationConfig {
    Kafka {
        brokers: String,
        group_id: String,
        ordered_topics: Vec<String>,
        unordered_topics: Vec<String>,
        task_limit: usize,
    },
    Amqp {
        consumer_tag: String,
        connection_string: String,
        ordered_queue_names: Vec<String>,
        unordered_queue_names: Vec<String>,
        task_limit: usize,
    },
    Grpc {
        grpc_port: u16,
        task_limit: usize,
        report_endpoint_url: Url,
    },
}

impl CommunicationConfig {
    pub fn task_limit(&self) -> usize {
        match self {
            CommunicationConfig::Kafka { task_limit, .. } => *task_limit,
            CommunicationConfig::Amqp { task_limit, .. } => *task_limit,
            CommunicationConfig::Grpc { task_limit, .. } => *task_limit,
        }
    }

    pub fn ordered_configs<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = CommonConsumerConfig<'a>> + 'a> {
        match self {
            CommunicationConfig::Kafka {
                brokers,
                group_id,
                ordered_topics,
                ..
            } => {
                let iter = ordered_topics
                    .iter()
                    .map(move |topic| CommonConsumerConfig::Kafka {
                        brokers: &brokers,
                        group_id: &group_id,
                        topic,
                    });
                Box::new(iter)
            }
            CommunicationConfig::Amqp {
                consumer_tag,
                connection_string,
                ordered_queue_names,
                ..
            } => {
                let options = Some(BasicConsumeOptions {
                    exclusive: true,
                    ..Default::default()
                });
                let iter =
                    ordered_queue_names
                        .iter()
                        .map(move |queue_name| CommonConsumerConfig::Amqp {
                            connection_string: &connection_string,
                            consumer_tag: &consumer_tag,
                            queue_name,
                            options,
                        });
                Box::new(iter)
            }
            CommunicationConfig::Grpc { .. } => Box::new(std::iter::empty()),
        }
    }

    pub fn unordered_configs<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = CommonConsumerConfig<'a>> + 'a> {
        match self {
            CommunicationConfig::Kafka {
                brokers,
                group_id,
                unordered_topics,
                ..
            } => {
                let iter = unordered_topics
                    .iter()
                    .map(move |topic| CommonConsumerConfig::Kafka {
                        brokers: &brokers,
                        group_id: &group_id,
                        topic,
                    });
                Box::new(iter)
            }
            CommunicationConfig::Amqp {
                consumer_tag,
                connection_string,
                unordered_queue_names,
                ..
            } => {
                let options = Some(BasicConsumeOptions {
                    exclusive: false,
                    ..Default::default()
                });
                let iter = unordered_queue_names.iter().map(move |queue_name| {
                    CommonConsumerConfig::Amqp {
                        connection_string: &connection_string,
                        consumer_tag: &consumer_tag,
                        queue_name,
                        options,
                    }
                });
                Box::new(iter)
            }
            CommunicationConfig::Grpc { grpc_port, .. } => {
                let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), *grpc_port);
                let iter = std::iter::once(CommonConsumerConfig::Grpc { addr });
                Box::new(iter)
            }
        }
    }
}
