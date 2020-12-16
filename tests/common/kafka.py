import json
from kafka import KafkaProducer
from contextlib import contextmanager


def push_to_kafka(producer, data, topic):
    producer.send(
        topic,
        json.dumps(data).encode(),
        key=data['objectId'].encode(),
        timestamp_ms=data['timestamp']
    ).get(3)


@contextmanager
def kafka_producer_manager():
    kafka_producer = KafkaProducer(bootstrap_servers='localhost:9092')
    try:
        yield kafka_producer
    finally:
        kafka_producer.close()
