import json
import pytest
import requests
import grpc
import tests.query_router.schema_registry_pb2 as pb2
import tests.query_router.schema_registry_pb2_grpc as pb2_grpc

from tests.common import load_case
from tests.common.cdl_env import CdlEnv
from tests.common.query_router import QueryRouter
from tests.common.config import PostgresConfig
from tests.common.postgres import connect_to_postgres, insert_test_data


def insert_test_metrics(data):
    line = "\n".join(data)
    requests.post("http://localhost:8428/write", line)


def registry_create_schema(url, name, topic, query, body, schema_type):
    with grpc.insecure_channel(url) as channel:
        stub = pb2_grpc.SchemaRegistryStub(channel)
        resp = stub.AddSchema(pb2.NewSchema(
            id="", name=name, topic=topic, query_address=query, definition=body, schema_type=schema_type))
        return resp.id


def query_get_single(url, schema_id, object_id, body):
    return requests.post(f"{url}/single/{object_id}", body, headers={'SCHEMA_ID': schema_id})


def query_get_multiple(url, schema_id, object_ids):
    return requests.get(f"{url}/multiple/{object_ids}", headers={'SCHEMA_ID': schema_id})


@pytest.fixture(params=['non_existing', 'single_schema', 'multiple_schemas'])
def prepare(request):
    with CdlEnv('.', postgres_config=PostgresConfig()) as env:
        data, expected = load_case(request.param, 'query_router')

        db = connect_to_postgres(env.postgres_config)

        with QueryRouter('1024', '50103', 'http://localhost:50101') as _:
            insert_test_data(db, data['database_setup'])

            sid = registry_create_schema('localhost:50101',
                                         'test_schema',
                                         'cdl.document.input',
                                         'http://localhost:50102',
                                         '{}',
                                         0)

            yield data, sid, expected

        db.close()


def test_endpoint_multiple(prepare):
    data, sid, expected = prepare

    # Request QR for data
    response = query_get_multiple(
        'http://localhost:50103', sid, data['query_for'])

    json1 = json.dumps(response.json(), sort_keys=True)
    json2 = json.dumps(expected, sort_keys=True)
    assert json1 == json2
    # assert response.json() == expected

    print(data)
    print(expected)


def test_endpoint_single_ds():
    with CdlEnv('.', postgres_config=PostgresConfig()) as env:
        data, expected = load_case('query_ds', 'query_router')

        with QueryRouter('1024', '50103', 'http://localhost:50101') as _:

            db = connect_to_postgres(env.postgres_config)
            insert_test_data(db, data['database_setup'])
            db.close()

            sid = registry_create_schema('localhost:50101',
                                         'test_schema',
                                         'cdl.document.input',
                                         'http://localhost:50102',
                                         '{}',
                                         0)

            # Request QR for data
            response = query_get_single(
                'http://localhost:50103', sid, data['query_for'], "{}")

            json1 = json.dumps(response.json(), sort_keys=True)
            json2 = json.dumps(expected, sort_keys=True)
            assert json1 == json2
            # assert response.json() == expected

            print(data)
            print(expected)


def test_endpoint_single_ts():
    with CdlEnv('.') as env:
        data, expected = load_case('query_ts', 'query_router')

        with QueryRouter('1024', '50103', 'http://localhost:50101') as _:

            insert_test_metrics(data['database_setup'])

            sid = registry_create_schema('localhost:50101',
                                         'test_schema',
                                         'cdl.document.input',
                                         'http://localhost:50104',
                                         '{}',
                                         1)

            # Line protocol requires timestamps in [ns]
            # Victoriametrics stores them internally in [ms]
            # but PromQL queries use "unix timestamps" which are in [s]
            start = 1608216910
            end = 1608216919
            step = 1
            req_body = {"from": str(start), "to": str(end), "step": str(step)}

            # print(req_body)

            # export = requests.get("http://localhost:8428/api/v1/export",
            #                       params={'match': '{__name__!=""}'})
            # print(export.text)

            # q = requests.get("http://localhost:8428/api/v1/query_range",
            #                  params={
            #                      'query': '{object_id="6793227c-1b5a-413c-b310-1a86dc2d3c78"}',
            #                      "start": start,
            #                      "end": end,
            #                      "step": step
            #                  })
            # print(q.text)

            # Request QR for data
            response = query_get_single(
                'http://localhost:50103', sid, data['query_for'], json.dumps(req_body))

            json1 = json.dumps(response.json(), sort_keys=True)
            json2 = json.dumps(expected, sort_keys=True)
            assert json1 == json2
            # assert response.json() == expected

            print(data)
            print(expected)
