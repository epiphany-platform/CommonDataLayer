use utils::messaging_system::consumer::{
    AmqpConsumerConfig, BasicConsumeOptions, CommonConsumerConfig, KafkaConsumerConfig,
};

pub enum CommunicationConfig {
    MessageQueue(MessageQueueConfig),
    GRpc(GRpcConfig),
}

#[derive(Clone, Debug)]
pub enum MessageQueueConfig {
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
}

impl MessageQueueConfig {
    pub fn task_limit(&self) -> usize {
        match self {
            MessageQueueConfig::Kafka { task_limit, .. } => *task_limit,
            MessageQueueConfig::Amqp { task_limit, .. } => *task_limit,
        }
    }

    pub fn ordered_configs<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = CommonConsumerConfig<'a>> + 'a> {
        match self {
            MessageQueueConfig::Kafka {
                brokers,
                group_id,
                ordered_topics,
                ..
            } => {
                let iter = ordered_topics.iter().map(move |topic| {
                    CommonConsumerConfig::Kafka(KafkaConsumerConfig {
                        brokers: &brokers,
                        group_id: &group_id,
                        topic,
                    })
                });
                Box::new(iter)
            }
            MessageQueueConfig::Amqp {
                consumer_tag,
                connection_string,
                ordered_queue_names,
                ..
            } => {
                let options = Some(BasicConsumeOptions {
                    exclusive: true,
                    ..Default::default()
                });
                let iter = ordered_queue_names.iter().map(move |queue_name| {
                    CommonConsumerConfig::Amqp(AmqpConsumerConfig {
                        connection_string: &connection_string,
                        consumer_tag: &consumer_tag,
                        queue_name,
                        options,
                    })
                });
                Box::new(iter)
            }
        }
    }

    pub fn unordered_configs<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = CommonConsumerConfig<'a>> + 'a> {
        match self {
            MessageQueueConfig::Kafka {
                brokers,
                group_id,
                unordered_topics,
                ..
            } => {
                let iter = unordered_topics.iter().map(move |topic| {
                    CommonConsumerConfig::Kafka(KafkaConsumerConfig {
                        brokers: &brokers,
                        group_id: &group_id,
                        topic,
                    })
                });
                Box::new(iter)
            }
            MessageQueueConfig::Amqp {
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
                    CommonConsumerConfig::Amqp(AmqpConsumerConfig {
                        connection_string: &connection_string,
                        consumer_tag: &consumer_tag,
                        queue_name,
                        options,
                    })
                });
                Box::new(iter)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct GRpcConfig {
    pub grpc_port: u16,
}
