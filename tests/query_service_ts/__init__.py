import grpc
import rpc.proto.query_service_ts_pb2 as pb2
import rpc.proto.query_service_ts_pb2_grpc as pb2_grpc
import pytest

from tests.common.victoria_metrics import VictoriaMetrics
from tests.common.config import VictoriaMetricsConfig
from tests.common.cdl_env import cdl_env
from tests.common.query_service_ts import QueryServiceTs

INPUT_PORT = '50104'


@pytest.fixture
def prepare_env():
    with cdl_env('../deployment/compose', victoria_metrics_config=VictoriaMetricsConfig()) as env:
        with QueryServiceTs(INPUT_PORT, env.victoria_metrics_config),\
                grpc.insecure_channel(f"localhost:{INPUT_PORT}") as channel:
            stub = pb2_grpc.QueryServiceTsStub(channel)
            db = VictoriaMetrics(env.victoria_metrics_config)
            yield db, stub
            db.clear_data_base()
