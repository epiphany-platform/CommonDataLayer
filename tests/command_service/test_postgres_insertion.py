import json

import pytest
from contextlib import closing
from kafka import KafkaProducer

from tests.common import load_case, retry_retrieve
from tests.common.cdl_env import cdl_env
from tests.common.command_service import CommandService
from tests.common.config import PostgresConfig, KafkaInputConfig
from tests.common.postgres import fetch_data_table, connect_to_postgres
from tests.common.kafka import push_to_kafka

TOPIC = "cdl.document.input"


@pytest.fixture(params=['single_insert', 'multiple_inserts'])
def prepare(request):
    with cdl_env('.', postgres_config=PostgresConfig(), kafka_input_config=KafkaInputConfig(TOPIC)) as env:
        data, expected = load_case(request.param, 'command_service/postgres')

        with closing(KafkaProducer(bootstrap_servers='localhost:9092')) as producer, CommandService(env.kafka_input_config, db_config=env.postgres_config) as _, closing(connect_to_postgres(env.postgres_config)) as db:
            yield db, producer, data, expected


def test_inserting(prepare):
    db, producer, data, expected = prepare

    for entry in data:
        push_to_kafka(producer, entry, TOPIC)
    producer.flush()

    actual, err = retry_retrieve(lambda: fetch_data_table(db), len(expected))

    assert err is None
    assert actual == expected
