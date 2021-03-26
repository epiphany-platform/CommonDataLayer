import json
import uuid

from kafka import KafkaAdminClient, KafkaProducer
from kafka.admin import NewTopic


class KafkaInputConfig:
    def __init__(self, topic, brokers='localhost:9092', group_id=None):
        self.topic = topic
        self.brokers = brokers
        self.group_id = group_id or str(uuid.uuid1())

    def to_dict(self):
        return {
            "KAFKA_BROKERS": self.brokers,
            "KAFKA_GROUP_ID": self.group_id,
            "UNORDERED_SOURCES": self.topic,
        }


def create_kafka_topic(config: KafkaInputConfig, topic):
    client = KafkaAdminClient(bootstrap_servers=config.brokers)
    client.create_topics([NewTopic(topic, 1, 1)])
    client.close()


def delete_kafka_topic(config: KafkaInputConfig, topic):
    client = KafkaAdminClient(bootstrap_servers=config.brokers)
    client.delete_topics([topic])
    client.close()


def push_to_kafka(kafka_config: KafkaInputConfig, data, **kwargs):
    producer = KafkaProducer(bootstrap_servers=kafka_config.brokers)

    object_id = (kwargs.get('key') or data['objectId']).encode()
    timestamp = kwargs.get('timestamp') or data['timestamp']

    producer.send(kafka_config.topic,
                  json.dumps(data).encode(),
                  key=object_id,
                  timestamp_ms=timestamp).get()

    producer.flush()
    producer.close()


class KafkaReportConfig:
    def __init__(self, topic='cdl.reports', brokers='localhost:9092'):
        self.topic = topic
        self.brokers = brokers

    def to_dict(self):
        return {
            "REPORT_DESTINATION": self.topic,
        }
