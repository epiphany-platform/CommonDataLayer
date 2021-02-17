import pytest

from tests.common.config import KafkaInputConfig, PostgresConfig
from tests.common.query_router import QueryRouter
from tests.common.query_service import QueryService
from tests.common.schema_registry import SchemaRegistry
from tests.common import load_case, assert_json
from tests.common.postgres import clear_data, insert_data

DB_NAME = 'query-router'
TOPIC = 'qr.test.multiple'


@pytest.fixture(params=['multiple/non_existing', 'multiple/single_schema', 'multiple/multiple_schemas'])
def prepare(request, tmp_path):
    data, expected = load_case(request.param, 'query_router')

    # declare environment
    kafka_input_config = KafkaInputConfig(TOPIC)
    postgres_config = PostgresConfig()

    qs = QueryService(db_config=postgres_config)
    sr = SchemaRegistry(str(tmp_path), kafka_input_config.brokers)

    # prepare environment
    clear_data(postgres_config)
    insert_data(postgres_config, data['database_setup'])

    qs.start()
    sr.start()

    schema_id = sr.create_schema('test', kafka_input_config.topic, f'http://localhost:{qs.input_port}', '{}', 0)

    with QueryRouter(f'http://localhost:{sr.input_port}') as qr:
        yield data, expected, qr, schema_id

    # cleanup environment
    clear_data(postgres_config)

    sr.stop()
    qs.stop()


def test_endpoint_multiple(prepare):
    data, expected, qr, schema_id = prepare

    response = qr.query_get_multiple(schema_id, data['query_for'])

    assert_json(response.json(), expected)
