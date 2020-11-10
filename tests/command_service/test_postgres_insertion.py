import json
import os
import subprocess
import psycopg2
import pytest

from kafka import KafkaProducer

from tests.common import load_case, retry_retrieve
from tests.common.postgres import fetch_data_table, clear_data_table

TOPIC = "cdl.document.input"

PSQL_URL = os.getenv("POSTGRES_CONNECTION_URL") or "postgresql://postgres:1234@localhost:5432/postgres"
EXECUTABLE = os.getenv("COMMAND_SERVICE_EXE") or "command-service"
KAFKA_BROKERS = os.getenv("KAFKA_BROKERS") or "localhost:9092"


def push_to_kafka(producer, data):
    producer\
        .send(TOPIC, json.dumps(data).encode(), key=data['object_id'].encode(), timestamp_ms=data['timestamp'])\
        .get(3)


@pytest.fixture(params=['single_insert', 'multiple_inserts'])
def prepare(request):
    svc = subprocess.Popen([EXECUTABLE, "postgres"],
                           env={"POSTGRES_OUTPUT_URL": PSQL_URL, "KAFKA_INPUT_BROKERS": KAFKA_BROKERS,
                                "KAFKA_INPUT_TOPIC": TOPIC,
                                "KAFKA_INPUT_GROUP_ID": "cdl.command-service.psql",
                                "REPORT_BROKER": KAFKA_BROKERS, "REPORT_TOPIC": "cdl.notify"})

    data, expected = load_case(request.param, "command_service")

    db = psycopg2.connect(PSQL_URL)

    yield db, data, expected

    svc.kill()

    clear_data_table(db)
    db.close()


def test_inserting(prepare):
    db, data, expected = prepare

    producer = KafkaProducer(bootstrap_servers=KAFKA_BROKERS)
    for entry in data:
        push_to_kafka(producer, entry)
    producer.flush()

    actual, err = retry_retrieve(lambda: fetch_data_table(db), len(expected))

    assert err is None
    assert actual == expected
