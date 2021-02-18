import time

import grpc
import pytest
import json
from tests.common import load_case, VictoriaMetricsConfig
from tests.common.query_service_ts import QueryServiceTs
from tests.common.victoria_metrics import clear_data, insert_data
from tests.rpc.proto import query_service_ts_pb2_grpc
from tests.rpc.proto.query_service_ts_pb2 import Range


@pytest.fixture
def prepare():
    data, expected = load_case('range/data', 'query_service_ts')

    # declare environment
    victoria_metrics_config = VictoriaMetricsConfig()

    # setup environment
    qs = QueryServiceTs(db_config=victoria_metrics_config)
    channel = grpc.insecure_channel(f"localhost:{qs.input_port}")
    stub = query_service_ts_pb2_grpc.QueryServiceTsStub(channel)

    setup_data = data['database_setup']
    start = int(time.time())
    end = start + len(setup_data)

    for i in range(0, len(setup_data)):
        ts = (start + i) * 1_000_000_000  # VM requires nanoseconds when inserting data via Influx LineProtocol
        setup_data[i] = setup_data[i].replace("$TIMESTAMP", str(ts))
        expected['data']['result'][0]['values'][i][0] = start + i

    clear_data(victoria_metrics_config)
    insert_data(victoria_metrics_config, setup_data)

    qs.start()

    query = data["query_for"]
    query['start'] = str(start)
    query['end'] = str(end)

    yield stub, expected, query

    # cleanup environment
    qs.stop()

    clear_data(victoria_metrics_config)


def test_query_by_range(prepare):
    stub, expected, query = prepare
    response = stub.QueryByRange(Range(**query))
    assert json.loads(str(response.timeseries)) == expected
