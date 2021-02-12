import json

from kafka import KafkaAdminClient, KafkaProducer
from kafka.admin import NewTopic

from tests.common.config import KafkaInputConfig


def create_kafka_topic(config: KafkaInputConfig, topic):
    client = KafkaAdminClient(bootstrap_servers=config.brokers)
    client.create_topics([NewTopic(topic, 1, 1)])
    client.close()


def delete_kafka_topic(config: KafkaInputConfig, topic):
    client = KafkaAdminClient(bootstrap_servers=config.brokers)
    client.delete_topics([topic])
    client.close()


def push_to_kafka(kafka_config: KafkaInputConfig, data):
    producer = KafkaProducer(bootstrap_servers=kafka_config.brokers)

    producer.send(
        kafka_config.topic,
        json.dumps(data).encode(),
        key=data['objectId'].encode(),
        timestamp_ms=data['timestamp']
    ).get()

    producer.flush()
    producer.close()
