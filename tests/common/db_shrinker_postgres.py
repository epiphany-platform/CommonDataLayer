import os
import subprocess

from tests.common.postgres import PostgresConfig

EXE = os.getenv('DB_SHRINKER_POSTGRES_EXE') or 'db-shrinker-postgres'


class DbShrinkerPostgres:
    def __init__(self, postgres_config: PostgresConfig):
        self.postgres_config = postgres_config

    def run(self):
        env = self.postgres_config.to_dict()
        env.update(
            RUST_LOG = "db_shrinker_postgres=trace,info",
        )
        svc = subprocess.Popen([EXE], env=env)
        svc.wait()
