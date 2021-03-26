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
                 communication_port=50110):
        self.communication_port = communication_port
        self.consumer_config = consumer_config
        self.postgres_config = postgres_config
        self.svc = None

    def start(self):
        env = self.postgres_config.to_dict()
        if type(self.consumer_config) is KafkaInputConfig:
            env.update(METHOD='kafka',
                       MQ_HOST=self.consumer_config.brokers,
                       MQ_TAG=self.consumer_config.group_id,
                       MQ_SOURCE=self.consumer_config.topic)
        else:
            raise Exception("Unsupported kind of consumer_config")
        env.update(COMMUNICATION_PORT='50110', METRICS_PORT='50104')

        self.svc = subprocess.Popen([EXE], env=env)

        time.sleep(3)

        return self

    def stop(self):
        self.svc.kill()
