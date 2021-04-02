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
            "RUST_LOG": "object_builder=trace,info",
            "INPUT_PORT": self.input_port,
            "METRICS_PORT": '50106',
            "STATUS_PORT": "0",
            "SCHEMA_REGISTRY_ADDR": self.schema_registry_addr
        }

        if type(self.consumer_config) is KafkaInputConfig:
            env.update(MQ_METHOD='kafka',
                       KAFKA_BROKERS=self.consumer_config.brokers,
                       KAFKA_GROUP_ID=self.consumer_config.group_id,
                       MQ_SOURCE=self.consumer_config.topic)
        else:
            raise Exception("Unsupported kind of consumer_config")

        self.svc = subprocess.Popen([EXE], env=env)

        time.sleep(3)

        return self

    def stop(self):
        self.svc.kill()
