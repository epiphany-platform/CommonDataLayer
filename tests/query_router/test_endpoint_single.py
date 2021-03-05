import json
import time

import pytest

from tests.common import load_case, assert_json
from tests.common.config import KafkaInputConfig, PostgresConfig, VictoriaMetricsConfig
from tests.common.postgres import clear_data as psql_clear_data, insert_data as psql_insert_data
from tests.common.query_router import QueryRouter
from tests.common.query_service import QueryService
from tests.common.query_service_ts import QueryServiceTs
from tests.common.schema_registry import SchemaRegistry
from tests.common.victoria_metrics import clear_data as vm_clear_data, insert_data as vm_insert_data

TOPIC = 'qr.test.single'


@pytest.fixture
def prepare_postgres(tmp_path):
    data, expected = load_case('single/query_ds', 'query_router')

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
    data, expected = load_case('single/query_ts', 'query_router')

    # declare environment
    kafka_input_config = KafkaInputConfig(TOPIC)
    victoria_metrics_config = VictoriaMetricsConfig()

    qs = QueryServiceTs(db_config=victoria_metrics_config)
    sr = SchemaRegistry(str(tmp_path), kafka_input_config.brokers)

    # prepare environment
    vm_clear_data(victoria_metrics_config)

    setup_data = data['database_setup']
    start = int(time.time())
    end = start + len(setup_data)

    for i in range(0, len(setup_data)):
        ts = (start + i) * 1_000_000_000  # VM requires nanoseconds when inserting data via Influx LineProtocol
        setup_data[i] = setup_data[i].replace("$TIMESTAMP", str(ts))
        expected['data']['result'][0]['values'][i][0] = start + i

    vm_insert_data(victoria_metrics_config, setup_data)

    qs.start()
    sr.start()

    schema_id = sr.create_schema('test', kafka_input_config.topic, f'http://localhost:{qs.input_port}', '{}', 1)

    with QueryRouter(f'http://localhost:{sr.input_port}') as qr:
        yield data, expected, start, end, qr, schema_id

    # cleanup environment
    qs.stop()
    sr.stop()

    vm_clear_data(victoria_metrics_config)


def test_endpoint_single_document_storage(prepare_postgres):
    data, expected, qr, schema_id = prepare_postgres

    response = qr.query_get_single(schema_id, data['query_for'], "{}")

    assert_json(response.json(), expected)


def test_endpoint_single_timeseries(prepare_victoria_metrics):
    data, expected, start_ts, end_ts, qr, schema_id = prepare_victoria_metrics

    req_body = {"from": str(start_ts), "to": str(end_ts), "step": '1'}

    response = qr.query_get_single(
        schema_id, data['query_for'], json.dumps(req_body))

    assert_json(response.json(), expected)
