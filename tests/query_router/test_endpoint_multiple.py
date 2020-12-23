import pytest

from tests.query_router import *
from tests.common import load_case
from tests.common.postgres import connect_to_postgres, insert_test_data


@pytest.fixture(params=['multiple/non_existing', 'multiple/single_schema', 'multiple/multiple_schemas'])
def prepare(request, prepare_document_storage_env):
    env, qr, sid = prepare_document_storage_env
    data, expected = load_case(request.param, 'query_router')

    db = connect_to_postgres(env.postgres_config)
    insert_test_data(db, data['database_setup'])
    db.close()

    yield qr, data, sid, expected


def test_endpoint_multiple(prepare):
    qr, data, sid, expected = prepare

    response = qr.query_get_multiple(sid, data['query_for'])

    assert_json(response.json(), expected)
