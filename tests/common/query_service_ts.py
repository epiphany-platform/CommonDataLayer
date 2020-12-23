import os
import subprocess

from tests.common.config import VictoriaMetricsConfig

EXE = os.getenv('QUERY_SERVICE_TS_EXE') or 'query-service-ts'


class QueryServiceTs:
    def __init__(self, input_port='50104', db_config=None):
        self.db_config = db_config
        self.input_port = input_port

    def __enter__(self):
        env = {}

        plugin = None
        if type(self.db_config) is VictoriaMetricsConfig:
            plugin = 'victoria'

        if not plugin:
            raise Exception('Unsupported database or no database at all')

        env.update(INPUT_PORT=self.input_port)
        env.update(self.db_config.to_dict())

        self.svc = subprocess.Popen([EXE, plugin], env=env)
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.svc.kill()
