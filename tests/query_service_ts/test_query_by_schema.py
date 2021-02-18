import grpc
import pytest
import json

from tests.common import load_case, VictoriaMetricsConfig, strip_timestamp
from tests.common.query_service_ts import QueryServiceTs
from tests.common.victoria_metrics import clear_data, insert_data
from tests.rpc.proto import query_service_ts_pb2_grpc
from tests.rpc.proto.query_service_ts_pb2 import SchemaId


@pytest.fixture(params=["schema/single", "schema/non_existing", "schema/invalid_schema_id"])
def prepare(request):
    data, expected = load_case(request.param, 'query_service_ts')

    # declare environment
    victoria_metrics_config = VictoriaMetricsConfig()

    # setup environment
    qs = QueryServiceTs(db_config=victoria_metrics_config)
    channel = grpc.insecure_channel(f"localhost:{qs.input_port}")
    stub = query_service_ts_pb2_grpc.QueryServiceTsStub(channel)

    clear_data(victoria_metrics_config)
    insert_data(victoria_metrics_config, data['database_setup'])

    qs.start()

    yield stub, expected, data["query_for"]

    # cleanup environment
    qs.stop()

    clear_data(victoria_metrics_config)


def test_query_by_schema(prepare):
    stub, expected, query = prepare

    response = stub.QueryBySchema(SchemaId(**query))

    assert strip_timestamp(json.loads(str(response.timeseries))) == expected
