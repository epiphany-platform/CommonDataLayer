import json
import os
import subprocess
import requests
import pytest

from time import sleep
from kafka import KafkaProducer

from tests.common import load_case, kafka

TOPIC = "cdl.timeseries.input"
KAFKA_BROKERS = os.getenv("KAFKA_BROKERS", "localhost:9092")
KAFKA_INPUT_GROUP_ID = "victoria_command"
KAFKA_INPUT_TOPIC = "cdl.timeseries.input"
VICTORIA_METRICS_URL = os.getenv(
    "VICTORIA_METRICS_URL", "http://0.0.0.0:8428")


def clear_data_table():
    delete_url = os.path.join(VICTORIA_METRICS_URL,
                              "api/v1/admin/tsdb/delete_series")
    requests.post(delete_url, data={"match[]": '{__name__!=""}'})


def fetch_data_table():
    export_url = os.path.join(VICTORIA_METRICS_URL,
                              "api/v1/export")
    json_lines = []
    for line in requests.get(export_url, 'match[]={__name__!=""}').text.splitlines():
        json_lines.append(json.loads(line))
    return json_lines


@pytest.fixture(params=['single_insert', 'multiple_inserts'])
def prepare(request):

    data, expected = load_case(
        request.param, "command_service/victoria_command")

    yield data, expected
    clear_data_table()


def test_inserting(prepare):
    data, expected = prepare

    producer = KafkaProducer(bootstrap_servers=KAFKA_BROKERS)
    for entry in data:
        kafka.push_to_kafka(producer, entry, TOPIC)
    producer.flush()
    # TODO: Actively wait for DB to update
    sleep(3)
    actual = fetch_data_table()
    assert actual == expected
