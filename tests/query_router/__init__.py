import json
import pytest

from tests.common.cdl_env import cdl_env
from tests.common.config import KafkaInputConfig, PostgresConfig, VictoriaMetricsConfig
from tests.common.query_router import QueryRouter
from tests.common.query_service import QueryService
from tests.common.query_service_ts import QueryServiceTs
from tests.common.schema_registry import SchemaRegistry


def assert_json(lhs, rhs):
    assert json.dumps(lhs, sort_keys=True) == json.dumps(rhs, sort_keys=True)


@pytest.fixture
def prepare_env(tmp_path):
    with cdl_env('../deployment/compose', kafka_input_config=KafkaInputConfig('cdl.data.input'),
                 postgres_config=PostgresConfig(),
                 victoria_metrics_config=VictoriaMetricsConfig()) as env:
        with SchemaRegistry(str(tmp_path), env.kafka_input_config.brokers) as sr:
            with QueryRouter(f'http://localhost:{sr.input_port}') as qr:
                yield env, qr, sr


@pytest.fixture
def prepare_document_storage_env(prepare_env):
    env, qr, sr = prepare_env
    with QueryService(db_config=env.postgres_config) as qs:
        sid = sr.create_schema('test_schema',
                               env.kafka_input_config.topic,
                               f'http://localhost:{qs.input_port}',
                               '{}',
                               0)
        yield env, qr, sid


@pytest.fixture
def prepare_timeseries_env(prepare_env):
    env, qr, sr = prepare_env
    with QueryServiceTs(db_config=env.victoria_metrics_config) as qs:
        sid = sr.create_schema('test_schema',
                               env.kafka_input_config.topic,
                               f'http://localhost:{qs.input_port}',
                               '{}',
                               1)
        yield env, qr, sid
