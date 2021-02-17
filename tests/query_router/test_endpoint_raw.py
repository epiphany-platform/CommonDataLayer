import json

import pytest

from tests.common import load_case, assert_json
from tests.common.config import KafkaInputConfig, PostgresConfig, VictoriaMetricsConfig
from tests.common.postgres import clear_data as psql_clear_data, insert_data as psql_insert_data
from tests.common.query_router import QueryRouter
from tests.common.query_service import QueryService
from tests.common.query_service_ts import QueryServiceTs
from tests.common.schema_registry import SchemaRegistry
from tests.common.victoria_metrics import clear_data as vm_clear_data, insert_data as vm_insert_data

TOPIC = 'qr.test.raw'


@pytest.fixture
def prepare_postgres(tmp_path):
    data, expected = load_case('raw/query_ds', 'query_router')

    # declare environment
    kafka_input_config = KafkaInputConfig(TOPIC)
    postgres_config = PostgresConfig()

    qs = QueryService(db_config=postgres_config)
    sr = SchemaRegistry(str(tmp_path), kafka_input_config.brokers)

    # prepare environment
    psql_clear_data(postgres_config)
    psql_insert_data(postgres_config, data['database_setup'])

    qs.start()
    sr.start()

    schema_id = sr.create_schema('test', kafka_input_config.topic, f'http://localhost:{qs.input_port}', '{}', 0)

    with QueryRouter(f'http://localhost:{sr.input_port}') as qr:
        yield data, expected, qr, schema_id

    # cleanup environment
    qs.stop()
    sr.stop()

    psql_clear_data(postgres_config)


@pytest.fixture
def prepare_victoria_metrics(tmp_path):
    data, expected = load_case('raw/query_ts', 'query_router')

    # declare environment
    kafka_input_config = KafkaInputConfig(TOPIC)
    victoria_metrics_config = VictoriaMetricsConfig()

    qs = QueryServiceTs(db_config=victoria_metrics_config)
    sr = SchemaRegistry(str(tmp_path), kafka_input_config.brokers)

    # prepare environment
    vm_clear_data(victoria_metrics_config)
    vm_insert_data(victoria_metrics_config, data['database_setup'])

    qs.start()
    sr.start()

    schema_id = sr.create_schema('test', kafka_input_config.topic, f'http://localhost:{qs.input_port}', '{}', 1)

    with QueryRouter(f'http://localhost:{sr.input_port}') as qr:
        yield data, expected, qr, schema_id

    # cleanup environment
    qs.stop()
    sr.stop()

    vm_clear_data(victoria_metrics_config)


def test_endpoint_raw_document_storage(prepare_postgres):
    data, expected, qr, schema_id = prepare_postgres

    req_body = {
        "raw_statement": f"SELECT '{data['query_for']}'"
    }

    response = qr.query_get_raw(
        schema_id, json.dumps(req_body))

    assert_json(response.json(), expected)


def test_endpoint_raw_timeseries(prepare_victoria_metrics):
    data, expected, qr, schema_id = prepare_victoria_metrics

    req_body = {
        "raw_statement": f"{{\"method\": \"{str(data['method'])}\", \"endpoint\": \"{str(data['endpoint'])}\", "
                         f"\"queries\": [[\"{str(data['queries'][0][0])}\", \"{str(data['queries'][0][1])}\"]] }}"
    }

    response = qr.query_get_raw(
        schema_id, json.dumps(req_body))

    assert response.json() == expected
