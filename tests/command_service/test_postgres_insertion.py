import json

import pytest
from kafka import KafkaProducer

from tests.common import load_case, retry_retrieve
from tests.common.cdl_env import CdlEnv
from tests.common.command_service import CommandService
from tests.common.config import PostgresConfig, KafkaInputConfig
from tests.common.postgres import fetch_data_table, connect_to_postgres

TOPIC = "cdl.document.input"


def push_to_kafka(producer, data):
    producer.send(
        TOPIC,
        json.dumps(data).encode(),
        key=data['objectId'].encode(),
        timestamp_ms=data['timestamp']
    ).get(3)


@pytest.fixture(params=['single_insert', 'multiple_inserts'])
def prepare(request):
    with CdlEnv('.', postgres_config=PostgresConfig(), kafka_input_config=KafkaInputConfig(TOPIC)) as env:
        data, expected = load_case(request.param, 'command_service')

        db = connect_to_postgres(env.postgres_config)
        producer = KafkaProducer(bootstrap_servers='localhost:9092')

        with CommandService(env.kafka_input_config, db_config=env.postgres_config) as _:
            yield db, producer, data, expected

        producer.close()
        db.close()


def test_inserting(prepare):
    db, producer, data, expected = prepare

    for entry in data:
        push_to_kafka(producer, entry)
    producer.flush()

    actual, err = retry_retrieve(lambda: fetch_data_table(db), len(expected))

    assert err is None
    assert actual == expected
