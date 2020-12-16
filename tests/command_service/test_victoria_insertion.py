import json
import os
import subprocess
import requests
import pytest

from time import sleep

from tests.common import load_case
from tests.common.kafka import kafka_producer_manager, push_to_kafka
from tests.common.victoria_metrics import VictoriaMetrics
from tests.common.cdl_env import cdl_env_manager
from tests.common.config import VictoriaMetricsConfig, KafkaInputConfig
from tests.common.command_service import CommandService


TOPIC = "cdl.timeseries.input"
VICTORIA_METRICS_URL = os.getenv(
    "VICTORIA_METRICS_URL", "http://127.0.0.1:12345")


@pytest.fixture(params=['single_insert', 'multiple_inserts'])
def prepare(request):

    with cdl_env_manager('.',  kafka_input_config=KafkaInputConfig(TOPIC), victoria_metrics_config=VictoriaMetricsConfig(VICTORIA_METRICS_URL)) as env:
        data, expected = load_case(
            request.param, "command_service/victoria_command")
        db = VictoriaMetrics(env.victoria_metrics_config)

        with kafka_producer_manager() as producer:
            with CommandService(env.kafka_input_config, db_config=env.victoria_metrics_config) as _:
                yield db, producer, data, expected
        db.clear_data_base()


def test_inserting(prepare):
    db, producer, data, expected = prepare

    for entry in data:
        push_to_kafka(producer, entry, TOPIC)
    producer.flush()
    # TODO: Actively wait for DB to update
    sleep(1)
    actual = db.fetch_data_table()
    for a in actual:
        assert a in expected
