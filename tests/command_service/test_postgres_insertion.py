import pytest

from tests.common import load_case, retry_retrieve
from tests.common.command_service import CommandService
from tests.common.config import KafkaInputConfig, PostgresConfig
from tests.common.kafka import push_to_kafka, create_kafka_topic, delete_kafka_topic
from tests.common.postgres import clear_data, fetch_data

TOPIC = "cdl.testing.command-service.postgres"


@pytest.fixture(params=['single_insert', 'multiple_inserts'])
def prepare(request):
    data, expected = load_case(request.param, 'command_service/postgres')

    topic = f'{TOPIC}.{request.param}'

    # declare environment
    kafka_config = KafkaInputConfig(topic)
    postgres_config = PostgresConfig()

    # prepare environment
    create_kafka_topic(kafka_config, topic)
    clear_data(postgres_config)

    with CommandService(kafka_config, db_config=postgres_config):
        yield data, expected, kafka_config, postgres_config

    # cleanup environment
    delete_kafka_topic(kafka_config, topic)
    clear_data(postgres_config)


def test_inserting(prepare):
    data, expected, kafka_config, postgres_config = prepare

    for entry in data:
        push_to_kafka(kafka_config, entry)

    actual, err = retry_retrieve(lambda: fetch_data(postgres_config), len(expected))

    assert err is None
    assert actual == expected
