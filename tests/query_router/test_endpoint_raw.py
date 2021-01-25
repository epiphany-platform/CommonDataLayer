import time
import pytest
import requests
from urllib.parse import urljoin

from tests.query_router import *
from tests.common import load_case
from tests.common.postgres import connect_to_postgres, insert_test_data


def test_endpoint_raw_document_storage(prepare_document_storage_env):
    env, qr, sid = prepare_document_storage_env
    data, expected = load_case('raw/query_ds', 'query_router')

    db = connect_to_postgres(env.postgres_config)
    insert_test_data(db, data['database_setup'])
    db.close()

    req_body = {
        "raw_statement": f"SELECT '{data['query_for']}'"
    }

    response = qr.query_get_raw(
        sid, json.dumps(req_body))

    assert_json(response.json(), expected)

def test_endpoint_raw_timeseries(prepare_timeseries_env):
    env, qr, sid = prepare_timeseries_env
    data, expected = load_case('raw/query_ts', 'query_router')

    lines = "\n".join(data['database_setup'])
    requests.post(
        urljoin(env.victoria_metrics_config.database_url, "/write"), lines)
    time.sleep(2)  # Ensure that 'search.latencyOffset' passed

    req_body = {
        "raw_statement": f"{{\"method\": \"{str(data['method'])}\", \"endpoint\": \"{str(data['endpoint'])}\", "
                         f"\"queries\": [[\"{str(data['queries'][0][0])}\", \"{str(data['queries'][0][1])}\"]] }}"
    }  # query must be well formatted

    response = qr.query_get_raw(
        sid, json.dumps(req_body))

    assert response.json() == expected
