import os
import subprocess
import time

import grpc
import tests.rpc.proto.schema_registry_pb2 as pb2
import tests.rpc.proto.schema_registry_pb2_grpc as pb2_grpc
from tests.common.postgres import PostgresConfig

EXE = os.getenv('SCHEMA_REGISTRY_EXE') or 'schema-registry'


class SchemaRegistry:
    def __init__(self,
                 edge_registry_addr,
                 kafka_brokers,
                 postgres_config: PostgresConfig,
                 kafka_group_id='schema_registry',
                 input_port='50101',
                 initial_schema=None):
        self.edge_registry_addr = edge_registry_addr
        self.kafka_brokers = kafka_brokers
        self.kafka_group_id = kafka_group_id
        self.input_port = input_port
        self.postgres_config = postgres_config
        self.initial_schema = initial_schema
        self.svc = None

    def start(self):
        env = {
            "SCHEMA_REGISTRY_COMMUNICATION_METHOD": 'kafka',
            "SCHEMA_REGISTRY_KAFKA__BROKERS": self.kafka_brokers,
            "SCHEMA_REGISTRY_KAFKA__GROUP_ID": self.kafka_group_id,
            "SCHEMA_REGISTRY_INPUT_PORT": self.input_port,
            "SCHEMA_REGISTRY_MONITORING__OTEL_SERVICE_NAME": 'schema-registry',
            "SCHEMA_REGISTRY_MONITORING__STATUS_PORT": '0',
            "SCHEMA_REGISTRY_SERVICES__EDGE_REGISTRY_URL": self.edge_registry_addr,
            **self.postgres_config.to_dict("SCHEMA_REGISTRY")
        }

        if self.initial_schema is not None:
            env.update(SCHEMA_REGISTRY_IMPORT_FILE=self.initial_schema)

        self.svc = subprocess.Popen([EXE], env=env)
        time.sleep(3)

        return self

    def stop(self):
        self.svc.kill()

    def create_schema(self, name, destination, query, body, schema_type):
        with grpc.insecure_channel(f"localhost:{self.input_port}") as channel:
            stub = pb2_grpc.SchemaRegistryStub(channel)
            resp = stub.AddSchema(
                pb2.NewSchema(
                    definition=bytes(body, 'utf-8'),
                    metadata=pb2.SchemaMetadata(name=name,
                                                insert_destination=destination,
                                                query_address=query,
                                                schema_type=pb2.SchemaType(schema_type=schema_type)),
                ))
            return resp.id
