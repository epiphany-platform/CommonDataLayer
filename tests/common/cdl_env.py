from testcontainers.compose import DockerCompose
import os
from contextlib import contextmanager

from tests.common import ensure_kafka_topic_exists, ensure_postgres_database_exists, ensure_victoria_metrics_database_exists
from tests.common.config import KafkaInputConfig, PostgresConfig, VictoriaMetricsConfig


class CdlEnv:
    def __init__(self,
                 testcontainers_path: os.path,
                 kafka_input_config: KafkaInputConfig = None,
                 postgres_config: PostgresConfig = None,
                 victoria_metrics_config: VictoriaMetricsConfig = None):
        self.testcontainers_path = testcontainers_path
        self.kafka_input_config = kafka_input_config
        self.postgres_config = postgres_config
        self.victoria_metrics_config = victoria_metrics_config

    def start(self):
        self.compose = DockerCompose(self.testcontainers_path)
        self.compose.start()

        if self.kafka_input_config:
            ensure_kafka_topic_exists(self.kafka_input_config)
        if self.postgres_config:
            ensure_postgres_database_exists(self.postgres_config)
        if self.victoria_metrics_config:
            ensure_victoria_metrics_database_exists(
                self.victoria_metrics_config)

        return self

    def stop(self):
        self.compose.stop()


@contextmanager
def cdl_env_manager(*args, **kwargs):
    cdl_env = CdlEnv(*args, **kwargs)
    try:
        yield cdl_env.start()
    finally:
        cdl_env.stop()
