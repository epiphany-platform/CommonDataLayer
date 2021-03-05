import time

import pytest

from tests.common import load_case, assert_json, retry_retrieve
from tests.common.command_service import CommandService
from tests.common.config import KafkaInputConfig, VictoriaMetricsConfig
from tests.common.kafka import push_to_kafka, create_kafka_topic, delete_kafka_topic
from tests.common.victoria_metrics import clear_data, fetch_data

TOPIC = "cdl.testing.command-service.victoria-metrics"


@pytest.fixture(params=['single_insert', 'multiple_inserts'])
def prepare(request):
    data, expected = load_case(request.param, "command_service/victoria_command")

    topic = f'{TOPIC}.{request.param}'

    # declare environment
    kafka_config = KafkaInputConfig(topic)
    victoria_metrics_config = VictoriaMetricsConfig()

    # prepare environment
    create_kafka_topic(kafka_config, topic)
    clear_data(victoria_metrics_config)

    with CommandService(kafka_config, db_config=victoria_metrics_config) as _:
        yield data, expected, kafka_config, victoria_metrics_config

    # cleanup environment
    delete_kafka_topic(kafka_config, topic)
    clear_data(victoria_metrics_config)


def test_inserting(prepare):
    data, expected, kafka_config, victoria_metrics_config = prepare

    for entry in data:
        push_to_kafka(kafka_config, entry)

    actual, err = retry_retrieve(lambda: fetch_data(victoria_metrics_config), len(expected))

    assert err is None
    assert_json(actual, expected)
