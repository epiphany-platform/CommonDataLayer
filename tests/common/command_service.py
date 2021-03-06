import os
import subprocess
from tests.common.victoria_metrics import VictoriaMetricsConfig
from tests.common.postgres import PostgresConfig

EXE = os.getenv('COMMAND_SERVICE_EXE') or 'command-service'


class CommandService:
    def __init__(self,
                 kafka_input_config,
                 kafka_report_config=None,
                 db_config=None):
        self.kafka_input_config = kafka_input_config
        self.kafka_report_config = kafka_report_config
        self.db_config = db_config

    def __enter__(self):
        env = self.kafka_input_config.to_dict("COMMAND_SERVICE")

        if self.kafka_report_config:
            env.update(self.kafka_report_config.to_dict("COMMAND_SERVICE"))
        env.update({'COMMAND_SERVICE_COMMUNICATION_METHOD': 'kafka'})
        plugin = None

        if type(self.db_config) is PostgresConfig:
            env.update({"COMMAND_SERVICE_REPOSITORY_KIND": 'postgres'})
            plugin = 'postgres'
        elif type(self.db_config) is VictoriaMetricsConfig:
            env.update({"COMMAND_SERVICE_REPOSITORY_KIND": 'victoria_metrics'})
            plugin = 'victoria-metrics'

        if not plugin:
            raise Exception('Unsupported database or no database at all')

        env.update(self.db_config.to_dict("COMMAND_SERVICE"))

        env.update({"COMMAND_SERVICE_MONITORING__OTEL_SERVICE_NAME": "command-service"})

        self.svc = subprocess.Popen([EXE, plugin], env=env)
        return self.svc

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.svc.kill()
