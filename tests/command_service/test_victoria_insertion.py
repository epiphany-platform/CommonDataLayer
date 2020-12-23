from contextlib import closing

import pytest
from kafka import KafkaProducer

from tests.common import load_case, retry_retrieve
from tests.common.cdl_env import cdl_env
from tests.common.command_service import CommandService
from tests.common.config import VictoriaMetricsConfig, KafkaInputConfig
from tests.common.kafka import push_to_kafka
from tests.common.victoria_metrics import VictoriaMetrics

TOPIC = "cdl.timeseries.input"


@pytest.fixture(params=['single_insert', 'multiple_inserts'])
def prepare(request):
    with cdl_env('.', kafka_input_config=KafkaInputConfig(TOPIC),
                 victoria_metrics_config=VictoriaMetricsConfig()) as env:
        data, expected = load_case(
            request.param, "command_service/victoria_command")
        db = VictoriaMetrics(env.victoria_metrics_config)

        with closing(KafkaProducer(bootstrap_servers='localhost:9092')) as producer, CommandService(
                env.kafka_input_config, db_config=env.victoria_metrics_config) as _:
            yield db, producer, data, expected
        db.clear_data_base()


def test_inserting(prepare):
    db, producer, data, expected = prepare

    for entry in data:
        push_to_kafka(producer, entry, TOPIC)
    producer.flush()
    actual, _ = retry_retrieve(db.fetch_data_table, len(expected))
    for a in actual:
        assert a in expected
