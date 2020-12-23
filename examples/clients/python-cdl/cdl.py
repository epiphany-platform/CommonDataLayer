from kafka import KafkaProducer
import grpc
import schema_registry_pb2 as pb2
import schema_registry_pb2_grpc as pb2_grpc
import requests


def kafka_insert_data(brokers, topic, payload):
    producer = KafkaProducer(bootstrap_servers=brokers)

    producer.send(topic, value=payload.encode(), key=b"cdl.test.data")

    producer.flush()
    producer.close()


def registry_create_schema(url, name, topic, query, body):
    with grpc.insecure_channel(url) as channel:
        stub = pb2_grpc.SchemaRegistryStub(channel)
        resp = stub.AddSchema(pb2.NewSchema(id="", name=name, topic=topic, query_address=query, definition=body))
        return resp.id


def query_get_single(url, schema_id, object_id):
    return requests.get(f"{url}/single/{object_id}", headers={'SCHEMA_ID': schema_id})
