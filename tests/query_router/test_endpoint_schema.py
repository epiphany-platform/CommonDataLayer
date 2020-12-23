import pytest

from tests.query_router import *
from tests.common import load_case
from tests.common.postgres import connect_to_postgres, insert_test_data


def test_endpoint_schema_ds(prepare_document_storage_env):
    env, qr, sid = prepare_document_storage_env
    data, expected = load_case('schema/query_ds', 'query_router')

    db = connect_to_postgres(env.postgres_config)
    for entry in data:
        entry['schema_id'] = sid
    insert_test_data(db, data)
    db.close()

    response = qr.query_get_schema(sid)

    assert_json(response.json(), expected)
