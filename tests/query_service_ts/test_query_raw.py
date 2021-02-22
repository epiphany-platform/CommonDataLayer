import grpc
import pytest
import json

from tests.common import load_case, VictoriaMetricsConfig, bytes_to_json
from tests.common.query_service_ts import QueryServiceTs
from tests.common.victoria_metrics import clear_data, insert_data
from tests.rpc.proto import query_service_ts_pb2_grpc
from tests.rpc.proto.query_service_pb2 import RawStatement


@pytest.fixture(params=["raw/export", "raw/get_query_range", "raw/post_query_range", "raw/handle_functions"])
def prepare(request):
    data, expected = load_case(request.param, "query_service_ts")

    # declare environment
    victoria_metrics_config = VictoriaMetricsConfig()

    # setup environment
    qs = QueryServiceTs(db_config=victoria_metrics_config)
    channel = grpc.insecure_channel(f"localhost:{qs.input_port}")
    stub = query_service_ts_pb2_grpc.QueryServiceTsStub(channel)

    clear_data(victoria_metrics_config)
    insert_data(victoria_metrics_config, data['database_setup'])

    qs.start()

    yield stub, expected, data['query_for']

    # cleanup environment
    qs.stop()

    clear_data(victoria_metrics_config)


def test_query_by_range(prepare):
    stub, expected, query = prepare
    response = stub.QueryRaw(RawStatement(
        raw_statement=json.dumps(query)))

    assert bytes_to_json(response.value_bytes) == expected


