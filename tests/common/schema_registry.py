import os
import subprocess
import time

import grpc
import rpc.proto.schema_registry_pb2 as pb2
import rpc.proto.schema_registry_pb2_grpc as pb2_grpc

EXE = os.getenv('SCHEMA_REGISTRY_EXE') or 'schema-registry'


class SchemaRegistry:
    def __init__(self,
                 db_name,
                 kafka_brokers,
                 kafka_group_id='schema_registry',
                 kafka_topics='cdl.schema_registry.internal',
                 replication_role='master',
                 input_port='50101'):
        self.db_name = db_name
        self.replication_role = replication_role
        self.kafka_brokers = kafka_brokers
        self.kafka_group_id = kafka_group_id
        self.kafka_topics = kafka_topics
        self.input_port = input_port

    def __enter__(self):
        env = {}

        env.update(DB_NAME=self.db_name)
        env.update(REPLICATION_ROLE=self.replication_role)
        env.update(REPLICATION_QUEUE='kafka')
        env.update(KAFKA_BROKERS=self.kafka_brokers)
        env.update(KAFKA_GROUP_ID=self.kafka_group_id)
        env.update(REPLICATION_TOPIC_OR_QUEUE=self.kafka_topics)
        env.update(REPLICATION_TOPIC_OR_EXCHANGE=self.kafka_topics)
        env.update(INPUT_PORT=self.input_port)

        self.svc = subprocess.Popen([EXE], env=env)
        time.sleep(1)

        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.svc.kill()

    def create_schema(self, name, topic, query, body, schema_type):
        with grpc.insecure_channel(f"localhost:{self.input_port}") as channel:
            stub = pb2_grpc.SchemaRegistryStub(channel)
            resp = stub.AddSchema(pb2.NewSchema(
                id="", name=name, topic=topic, query_address=query, definition=body, schema_type=schema_type))
            return resp.id
