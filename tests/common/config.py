import uuid
from urllib.parse import urljoin


class PostgresConfig:
    def __init__(self, user='postgres', password='1234', host='localhost', port='5432', dbname='postgres',
                 schema='cdl'):
        self.user = user
        self.password = password
        self.host = host
        self.port = port
        self.dbname = dbname
        self.schema = schema

    def to_dict(self):
        return {
            "POSTGRES_USERNAME": self.user,
            "POSTGRES_PASSWORD": self.password,
            "POSTGRES_HOST": self.host,
            "POSTGRES_PORT": self.port,
            "POSTGRES_DBNAME": self.dbname,
            "POSTGRES_SCHEMA": self.schema,
        }


class VictoriaMetricsConfig:
    def __init__(self, database_url="http://localhost:8428"):
        self.database_url = database_url

    def to_dict(self):
        return {
            "VICTORIA_METRICS_OUTPUT_URL": self.database_url,
            "VICTORIA_QUERY_URL": urljoin(self.database_url, '/api/v1'),
        }


class KafkaInputConfig:
    def __init__(self, topic, brokers='localhost:9092', group_id=None):
        self.topic = topic
        self.brokers = brokers
        self.group_id = group_id or str(uuid.uuid1())

    def to_dict(self):
        return {
            "KAFKA_BROKERS": self.brokers,
            "KAFKA_GROUP_ID": self.group_id,
            "UNORDERED_SOURCES": self.topic,
        }


class KafkaReportConfig:
    def __init__(self, topic='cdl.reports', brokers='localhost:9092'):
        self.topic = topic
        self.brokers = brokers

    def to_dict(self):
        return {
            "REPORT_DESTINATION": self.topic,
        }
