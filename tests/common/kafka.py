import json


def push_to_kafka(producer, data, topic):
    producer.send(
        topic,
        json.dumps(data).encode(),
        key=data['objectId'].encode(),
        timestamp_ms=data['timestamp']
    ).get(3)
