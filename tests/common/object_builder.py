import os
import subprocess
import time

from tests.common.kafka import KafkaInputConfig
from tests.common.postgres import PostgresConfig

EXE = os.getenv('OBJECT_BUILDER_EXE') or 'object-builder'


class ObjectBuilder:
    def __init__(self,
                 schema_registry_addr,
                 consumer_config,
                 input_port='50110'):
        self.input_port = input_port
        self.consumer_config = consumer_config
        self.schema_registry_addr = schema_registry_addr
        self.svc = None

    def start(self):
        env = {
            "OBJECT_BUILDER_INPUT_PORT": self.input_port,
            "OBJECT_BUILDER_SERVICES__SCHEMA_REGISTRY_URL": self.schema_registry_addr,
            "OBJECT_BUILDER_MONITORING__OTEL_SERVICE_NAME": 'object-builder',
            "OBJECT_BUILDER_CHUNK_CAPACITY": '100'
        }

        if type(self.consumer_config) is KafkaInputConfig:
            env.update(OBJECT_BUILDER_COMMUNICATION_METHOD='kafka',
                       OBJECT_BUILDER_KAFKA__BROKERS=self.consumer_config.brokers,
                       OBJECT_BUILDER_KAFKA__GROUP_ID=self.consumer_config.group_id,
                       OBJECT_BUILDER_KAFKA__INGEST_TOPIC=self.consumer_config.topic)
        else:
            raise Exception("Unsupported kind of consumer_config")

        self.svc = subprocess.Popen([EXE], env=env)

        time.sleep(3)

        return self

    def stop(self):
        self.svc.kill()
