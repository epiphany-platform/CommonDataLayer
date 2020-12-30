import pytest
import json
import time

from tests.query_service_ts import prepare_env
from tests.common import load_case
from rpc.proto.query_service_ts_pb2 import SchemaId


@pytest.fixture(params=["schema/single"])
def prepare(request, prepare_env):
    db, stub = prepare_env
    data, expected = load_case(request.param, "query_service_ts")
    db.insert_test_data(data['database_setup'])
    time.sleep(2)  # Ensure that 'search.latencyOffset' passed

    query = data["query_for"]
    return db, stub, expected, query

# TODO: Handle that instant query returns current timestamp instead of timestamp of insert


def test_query_by_schema(prepare):
    db, stub, expected, query = prepare
    print(db.fetch_data_table())
    query_request = SchemaId(**query)
    print(query_request)
    response = stub.QueryBySchema(query_request)
    print(response)
    assert json.loads(str(response.timeseries)) == expected
