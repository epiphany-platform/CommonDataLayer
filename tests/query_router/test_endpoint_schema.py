import pytest

from tests.common import load_case, assert_json
from tests.common.config import KafkaInputConfig, PostgresConfig
from tests.common.postgres import clear_data, insert_data
from tests.common.query_router import QueryRouter
from tests.common.query_service import QueryService
from tests.common.schema_registry import SchemaRegistry

TOPIC = 'qr.test.schema'

@pytest.fixture
def prepare(tmp_path):
    data, expected = load_case('schema/query_ds', 'query_router')

    # declare environment
    kafka_input_config = KafkaInputConfig(TOPIC)
    postgres_config = PostgresConfig()

    qs = QueryService(db_config=postgres_config)
    sr = SchemaRegistry(str(tmp_path), kafka_input_config.brokers)

    # prepare environment
    sr.start()

    schema_id = sr.create_schema('test', kafka_input_config.topic, f'http://localhost:{qs.input_port}', '{}', 0)

    clear_data(postgres_config)

    for entry in data:
        entry['schema_id'] = schema_id

    insert_data(postgres_config, data)

    qs.start()

    with QueryRouter(f'http://localhost:{sr.input_port}') as qr:
        yield expected, qr, schema_id

    # cleanup environment
    clear_data(postgres_config)

    sr.stop()
    qs.stop()


def test_endpoint_schema_ds(prepare):
    expected, qr, schema_id = prepare

    response = qr.query_get_schema(schema_id)

    assert_json(response.json(), expected)
