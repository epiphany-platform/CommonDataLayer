from testcontainers.compose import DockerCompose

from tests.common import ensure_kafka_topic_exists, ensure_postgres_database_exists


class CdlEnv:
    def __init__(self, testcontainers_path, kafka_input_config = None, postgres_config = None):
        self.testcontainers_path = testcontainers_path
        self.kafka_input_config = kafka_input_config
        self.postgres_config = postgres_config

    def __enter__(self):
        self.compose = DockerCompose(self.testcontainers_path)
        self.compose.start()

        if self.kafka_input_config:
            ensure_kafka_topic_exists(self.kafka_input_config)
        if self.postgres_config:
            ensure_postgres_database_exists(self.postgres_config)

        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.compose.stop()
