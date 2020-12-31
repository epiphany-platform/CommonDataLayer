import pytest
import json
import time

from tests.query_service_ts import prepare_env
from tests.common import load_case
from rpc.proto.query_service_ts_pb2 import SchemaId


@pytest.fixture(params=["schema/single", "schema/non_existing", "schema/invalid_schema_id"])
def prepare(request, prepare_env):
    db, stub = prepare_env
    data, expected = load_case(request.param, "query_service_ts")
    db.insert_test_data(data['database_setup'])

    query = data["query_for"]
    return db, stub, expected, query


def test_query_by_schema(prepare):
    db, stub, expected, query = prepare

    query_request = SchemaId(**query)
    response = stub.QueryBySchema(query_request)

    assert_without_timestamp(json.loads(str(response.timeseries)), expected)


# Instant query returns current timestamp instead of timestamp of data insert
def assert_without_timestamp(lhs, rhs):
    try:
        rhs['data']['result'][0]['value'][0] = lhs['data']['result'][0]['value'][0]
    except IndexError:
        pass

    assert lhs == rhs
