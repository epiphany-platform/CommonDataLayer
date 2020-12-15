import json
import os
import subprocess
import psycopg2
import pytest

from kafka import KafkaProducer

from tests.common import load_case, retry_retrieve, kafka
from tests.common.postgres import fetch_data_table, clear_data_table

TOPIC = "cdl.document.input"

POSTGRES_USERNAME = os.getenv("POSTGRES_USERNAME") or "postgres"
POSTGRES_PASSWORD = os.getenv("POSTGRES_PASSWORD") or "1234"
POSTGRES_HOST = os.getenv("POSTGRES_HOST") or "localhost"
POSTGRES_PORT = os.getenv("POSTGRES_PORT") or "5432"
POSTGRES_DBNAME = os.getenv("POSTGRES_DBNAME") or "postgres"
EXECUTABLE = os.getenv("COMMAND_SERVICE_EXE") or "command-service"
KAFKA_BROKERS = os.getenv("KAFKA_BROKERS") or "localhost:9092"

@pytest.fixture(params=['single_insert', 'multiple_inserts'])
def prepare(request):
    psql_url = f"postgresql://{POSTGRES_USERNAME}:{POSTGRES_PASSWORD}@{POSTGRES_HOST}:{POSTGRES_PORT}/{POSTGRES_DBNAME}"

    svc = subprocess.Popen([EXECUTABLE, "postgres"],
                           env={"POSTGRES_USERNAME": POSTGRES_USERNAME, "POSTGRES_PASSWORD": POSTGRES_PASSWORD,
                                "POSTGRES_HOST": POSTGRES_HOST, "POSTGRES_PORT": POSTGRES_PORT,
                                "POSTGRES_DBNAME": POSTGRES_DBNAME, "KAFKA_INPUT_BROKERS": KAFKA_BROKERS,
                                "KAFKA_INPUT_TOPIC": TOPIC, "KAFKA_INPUT_GROUP_ID": "cdl.command-service.psql",
                                "REPORT_BROKER": KAFKA_BROKERS, "REPORT_TOPIC": "cdl.notify"})

    data, expected = load_case(request.param, "command_service/postgres")

    db = psycopg2.connect(psql_url)

    yield db, data, expected

    svc.kill()

    clear_data_table(db)
    db.close()


def test_inserting(prepare):
    db, data, expected = prepare

    producer = KafkaProducer(bootstrap_servers=KAFKA_BROKERS)
    for entry in data:
        kafka.push_to_kafka(producer, entry, TOPIC)
    producer.flush()

    actual, err = retry_retrieve(lambda: fetch_data_table(db), len(expected))

    assert err is None
    assert actual == expected
