import os
import subprocess
import time

from tests.common.config import VictoriaMetricsConfig

EXE = os.getenv('QUERY_SERVICE_TS_EXE') or 'query-service-ts'


class QueryServiceTs:
    def __init__(self, input_port='50104', db_config=None):
        self.db_config = db_config
        self.input_port = input_port
        self.svc = None

    def start(self):
        plugin = None
        if type(self.db_config) is VictoriaMetricsConfig:
            plugin = 'victoria'

        if not plugin:
            raise Exception('Unsupported database or no database at all')

        env = self.db_config.to_dict()
        env.update(INPUT_PORT=self.input_port)
        env.update(METRICS_PORT="59104")

        self.svc = subprocess.Popen([EXE, plugin], env=env)

        time.sleep(3)

        return self

    def stop(self):
        self.svc.kill()
