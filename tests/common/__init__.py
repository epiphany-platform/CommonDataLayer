import json
import os
import time

from kafka import KafkaAdminClient
from kafka.admin import NewTopic
from kafka.errors import UnrecognizedBrokerVersion
from psycopg2._psycopg import OperationalError
from requests import HTTPError
from tests.common.victoria_metrics import VictoriaMetrics
from tests.common.postgres import connect_to_postgres
from tests.common.config import VictoriaMetricsConfig


def load_case(case_name, app):
    with open(os.path.join(os.path.dirname(os.path.realpath(__file__)), '..', 'data', app, f'{case_name}.json')) as f:
        json_document = json.load(f)
        return json_document['data'], json_document['expected']


def retry_retrieve(fetch, expected_rows, retries=10, delay=6):
    for _ in range(1, retries):
        data = fetch()

        if len(data) == expected_rows:
            return data, None

        time.sleep(delay)

    return None, 'Reached maximum number of retries'


def ensure_kafka_topic_exists(kafka_config):
    set_up = False
    for _ in range(1, 12):
        try:
            admin_client = KafkaAdminClient(bootstrap_servers=kafka_config.brokers)
            admin_client.create_topics([NewTopic(kafka_config.topic, 1, 1)])
            set_up = True
            break
        except (UnrecognizedBrokerVersion, ValueError):
            time.sleep(5)

    if not set_up:
        raise Exception('Failed to setup kafka topic')


def ensure_postgres_database_exists(postgres_config):
    set_up = False
    for _ in range(1, 12):
        try:
            db = connect_to_postgres(postgres_config)
            curr = db.cursor()
            curr.execute("CREATE SCHEMA cdl")
            curr.execute(
                """CREATE TABLE IF NOT EXISTS cdl.data (
                    object_id UUID NOT NULL,
                    version BIGINT NOT NULL,
                    schema_id UUID NOT NULL,
                    payload JSON NOT NULL,
                    PRIMARY KEY (object_id, version)
                )"""
            )
            db.commit()
            curr.close()
            db.close()
            set_up = True
            break
        except OperationalError:
            time.sleep(5)

    if not set_up:
        raise Exception('Failed to set up postgres database')


def ensure_victoria_metrics_database_exists(victoria_config):
    set_up = False
    for _ in range(1, 12):
        try:
            db = VictoriaMetrics(victoria_config)
            set_up = db.is_db_alive()
            if set_up:
                break
        except HTTPError:
            time.sleep(5)

    if not set_up:
        raise Exception('Failed to set up victoriametrics database')
