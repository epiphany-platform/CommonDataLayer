import os
import subprocess
import time

from tests.common.postgres import PostgresConfig

EXE = os.getenv('QUERY_SERVICE_EXE') or 'query-service'


class QueryService:
    def __init__(self, input_port='50102', db_config=None):
        self.db_config = db_config
        self.input_port = input_port
        self.svc = None

    def start(self):
        plugin = None
        if type(self.db_config) is PostgresConfig:
            plugin = 'postgres'

        if not plugin:
            raise Exception('Unsupported database or no database at all')

        env = self.db_config.to_dict("QUERY_SERVICE")

        env.update(QUERY_SERVICE_INPUT_PORT=self.input_port,
                   QUERY_SERVICE_MONITORING__OTEL_SERVICE_NAME='query-service',
                   QUERY_SERVICE_MONITORING__STATUS_PORT='0')

        self.svc = subprocess.Popen([EXE, plugin], env=env)

        time.sleep(3)

        return self

    def stop(self):
        self.svc.kill()
