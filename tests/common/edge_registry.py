import os
import subprocess
import time

from tests.common.kafka import KafkaInputConfig
from tests.common.postgres import PostgresConfig

EXE = os.getenv('EDGE_REGISTRY_EXE') or 'edge-registry'


class EdgeRegistry:
    def __init__(self,
                 consumer_config,
                 postgres_config: PostgresConfig,
                 rpc_port='50110'):
        self.rpc_port = rpc_port
        self.consumer_config = consumer_config
        self.postgres_config = postgres_config
        self.svc = None

    def start(self):
        env = self.postgres_config.to_dict("EDGE_REGISTRY")
        env.update(EDGE_REGISTRY_monitoring__otel_service_name='edge-registry')

        if type(self.consumer_config) is KafkaInputConfig:
            env.update(EDGE_REGISTRY_COMMUNICATION_METHOD='kafka',
                       EDGE_REGISTRY_KAFKA__BROKERS=self.consumer_config.brokers,
                       EDGE_REGISTRY_KAFKA__GROUP_ID=self.consumer_config.group_id,
                       EDGE_REGISTRY_KAFKA__INGEST_TOPIC=self.consumer_config.topic)
        else:
            raise Exception("Unsupported kind of consumer_config")
        env.update(EDGE_REGISTRY_INPUT_PORT=self.rpc_port)

        self.svc = subprocess.Popen([EXE], env=env)

        time.sleep(3)

        return self

    def stop(self):
        self.svc.kill()
