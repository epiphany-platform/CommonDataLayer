from testcontainers.compose import DockerCompose
import os
from contextlib import contextmanager

from tests.common import ensure_kafka_topic_exists, ensure_postgres_database_exists, ensure_victoria_metrics_database_exists
from tests.common.config import KafkaInputConfig, PostgresConfig, VictoriaMetricsConfig

CWD = os.getenv("WORKDIR") or os.getcwd()


class CdlEnvCofig:
    def __init__(self, testcontainers_path: os.path,
                 kafka_input_config: KafkaInputConfig = None,
                 postgres_config: PostgresConfig = None,
                 victoria_metrics_config: VictoriaMetricsConfig = None):
        self.testcontainers_path = testcontainers_path
        self.kafka_input_config = kafka_input_config
        self.postgres_config = postgres_config
        self.victoria_metrics_config = victoria_metrics_config


@ contextmanager
def cdl_env(
        testcontainers_path: os.path,
        kafka_input_config: KafkaInputConfig = None,
        postgres_config: PostgresConfig = None,
        victoria_metrics_config: VictoriaMetricsConfig = None):
    with DockerCompose(os.path.join(CWD, testcontainers_path)) as _:
        if kafka_input_config:
            ensure_kafka_topic_exists(kafka_input_config)
        if postgres_config:
            ensure_postgres_database_exists(postgres_config)
        if victoria_metrics_config:
            ensure_victoria_metrics_database_exists(victoria_metrics_config)

        yield CdlEnvCofig(testcontainers_path, kafka_input_config, postgres_config, victoria_metrics_config)
