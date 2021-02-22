import json
import os
import time
from tests.common.postgres import connect_to_postgres
from tests.common.config import VictoriaMetricsConfig


def load_case(case_name, app):
    with open(os.path.join(os.path.dirname(os.path.realpath(__file__)), '..', 'data', app, f'{case_name}.json')) as f:
        json_document = json.load(f)
        return json_document['data'], json_document['expected']


def retry_retrieve(fetch, expected_rows, retries=10, delay=6):
    for _ in range(1, retries):
        data = fetch()

        if len(data) == expected_rows:
            return data, None

        time.sleep(delay)

    return None, 'Reached maximum number of retries'


def assert_json(lhs, rhs):
    assert json.dumps(lhs, sort_keys=True) == json.dumps(rhs, sort_keys=True)


def bytes_to_json(b):
    return json.loads(b.decode("utf-8"))


def strip_timestamp(arr):
    for i in range(0, len(arr['data']['result'])):
        arr['data']['result'][i]['value'].pop(0)

    return arr
