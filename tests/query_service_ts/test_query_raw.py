import pytest
import json
from tests.query_service_ts import prepare_env
from tests.common import load_case
from rpc.proto.query_service_ts_pb2 import RawStatement


@pytest.fixture(params=["raw/export"])
def prepare(request, prepare_env):
    db, stub = prepare_env
    data, expected = load_case(request.param, "query_service_ts")
    db.insert_test_data(data['database_setup'])
    query = data["query_for"]
    return stub, expected, query


def test_query_by_range(prepare):
    stub, expected, query = prepare
    response = stub.QueryRaw(RawStatement(
        raw_statement=json.dumps(query)))

    assert bytes_to_json(response.value_bytes) == expected


def bytes_to_json(b):
    return json.loads(b.decode("utf-8"))
