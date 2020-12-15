import json
import os
import time

from kafka import KafkaAdminClient
from kafka.admin import NewTopic
from kafka.errors import UnrecognizedBrokerVersion
from psycopg2._psycopg import OperationalError

from tests.common.postgres import connect_to_postgres


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


def ensure_kafka_topic_exists(config):
    set_up = False
    for i in range(1, 12):
        try:
            admin_client = KafkaAdminClient(bootstrap_servers=config.brokers)
            admin_client.create_topics([NewTopic(config.topic, 1, 1)])
            set_up = True
            break
        except (UnrecognizedBrokerVersion, ValueError):
            time.sleep(5)

    if not set_up:
        raise Exception('Failed it setup kafka topic')


def ensure_postgres_database_exists(config):
    set_up = False
    for i in range(1, 12):
        try:
            db = connect_to_postgres(config)
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
