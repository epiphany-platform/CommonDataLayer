import time
import pytest
import requests
from urllib.parse import urljoin

from tests.query_router import *
from tests.common import load_case
from tests.common.postgres import connect_to_postgres, insert_test_data


def test_endpoint_single_document_storage(prepare_document_storage_env):
    env, qr, sid = prepare_document_storage_env
    data, expected = load_case('single/query_ds', 'query_router')

    db = connect_to_postgres(env.postgres_config)
    insert_test_data(db, data['database_setup'])
    db.close()

    response = qr.query_get_single(sid, data['query_for'], "{}")

    assert_json(response.json(), expected)


def test_endpoint_single_timeseries(prepare_timeseries_env):
    env, qr, sid = prepare_timeseries_env
    data, expected = load_case('single/query_ts', 'query_router')

    lines = "\n".join(data['database_setup'])
    requests.post(
        urljoin(env.victoria_metrics_config.database_url, "/write"), lines)
    time.sleep(2)  # Ensure that 'search.latencyOffset' passed

    # Line protocol requires timestamps in [ns]
    # Victoriametrics stores them internally in [ms]
    # but PromQL queries use "unix timestamps" which are in [s]
    start = 1608216910
    end = 1608216919
    step = 1
    req_body = {"from": str(start), "to": str(
        end), "step": str(step)}

    response = qr.query_get_single(
        sid, data['query_for'], json.dumps(req_body))

    assert_json(response.json(), expected)
