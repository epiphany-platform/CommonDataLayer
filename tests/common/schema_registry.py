import os
import subprocess
import time

import grpc
import tests.rpc.proto.schema_registry_pb2 as pb2
import tests.rpc.proto.schema_registry_pb2_grpc as pb2_grpc

EXE = os.getenv('SCHEMA_REGISTRY_EXE') or 'schema-registry'


class SchemaRegistry:
    def __init__(self,
                 db_name,
                 kafka_brokers,
                 kafka_group_id='schema_registry',
                 kafka_topics='cdl.schema_registry.internal',
                 replication_role='master',
                 input_port='50101',
                 initial_schema=None):
        self.db_name = db_name
        self.replication_role = replication_role
        self.kafka_brokers = kafka_brokers
        self.kafka_group_id = kafka_group_id
        self.kafka_topics = kafka_topics
        self.input_port = input_port
        self.initial_schema = initial_schema
        self.svc = None

    def start(self):
        env = {
            "DB_NAME": self.db_name,
            "REPLICATION_ROLE": self.replication_role,
            "COMMUNICATION_METHOD": 'kafka',
            "KAFKA_BROKERS": self.kafka_brokers,
            "KAFKA_GROUP_ID": self.kafka_group_id,
            "REPLICATION_SOURCE": self.kafka_topics,
            "REPLICATION_DESTINATION": self.kafka_topics,
            "INPUT_PORT": self.input_port,
            "METRICS_PORT": "59101",
            "STATUS_PORT": "0",
            "RUST_LOG": "schema_registry=trace,info",
        }

        if self.initial_schema is not None:
            env.update(IMPORT_FILE=self.initial_schema)

        self.svc = subprocess.Popen([EXE], env=env)
        time.sleep(3)

        return self

    def stop(self):
        self.svc.kill()

    def create_schema(self, name, destination, query, body, schema_type):
        with grpc.insecure_channel(f"localhost:{self.input_port}") as channel:
            stub = pb2_grpc.SchemaRegistryStub(channel)
            resp = stub.AddSchema(
                pb2.NewSchema(id="",
                              name=name,
                              insert_destination=destination,
                              query_address=query,
                              definition=body,
                              schema_type=schema_type))
            return resp.id
