import os
import subprocess
import time

import requests

EXE = os.getenv('QUERY_ROUTER_EXE') or 'query-router'


class QueryRouter:
    def __init__(self, schema_registry_addr, cache_capacity='1024', input_port='50103'):
        self.cache_capacity = cache_capacity
        self.input_port = input_port
        self.schema_registry_addr = schema_registry_addr

    def __enter__(self):
        env = {"CACHE_CAPACITY": self.cache_capacity, "INPUT_PORT": self.input_port, "METRICS_PORT": "59103",
               "SCHEMA_REGISTRY_ADDR": self.schema_registry_addr}

        self.svc = subprocess.Popen([EXE], env=env)
        time.sleep(3)

        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.svc.kill()

    def query_get_single(self, schema_id, object_id, body):
        return requests.post(f"http://localhost:{self.input_port}/single/{object_id}", body,
                             headers={'SCHEMA_ID': schema_id})

    def query_get_multiple(self, schema_id, object_ids):
        return requests.get(f"http://localhost:{self.input_port}/multiple/{object_ids}",
                            headers={'SCHEMA_ID': schema_id})

    def query_get_schema(self, schema_id):
        return requests.get(f"http://localhost:{self.input_port}/schema", headers={'SCHEMA_ID': schema_id})

    def query_get_raw(self, schema_id, body):
        return requests.post(f"http://localhost:{self.input_port}/raw", body, headers={'SCHEMA_ID': schema_id})
