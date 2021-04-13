import grpc
import pytest

from google.protobuf.json_format import MessageToDict
from tests.common import load_case_ext, assert_json
from tests.common.schema_registry import SchemaRegistry
from tests.common.object_builder import ObjectBuilder
from tests.common.query_service import QueryService
from tests.common.kafka import KafkaInputConfig, create_kafka_topic, delete_kafka_topic
from tests.rpc.proto import object_builder_pb2_grpc
from tests.rpc.proto.object_builder_pb2 import ViewId
from tests.rpc.proto.common_pb2 import MaterializedView, RowDefinition
from tests.common.postgres import clear_data, insert_data, PostgresConfig

TOPIC = "cdl.object_builder.tests_data"


@pytest.fixture(params=['simple', 'missing_view'])
def prepare(request, tmp_path):
    case = load_case_ext(f"rpc/{request.param}", 'object_builder')
    expected = case.get('expected', None)
    expectedError = case.get('expected_error', None)
    data = case['data']
    viewId = case['view_id']

    # declare environment
    postgres_config = PostgresConfig()
    kafka_config = KafkaInputConfig(TOPIC)

    # prepare environment
    clear_data(postgres_config)
    create_kafka_topic(kafka_config, TOPIC)

    insert_data(postgres_config, data)

    sr = SchemaRegistry(str(tmp_path), kafka_config.brokers, initial_schema = "data/object_builder/initial-schema.kafka.json")
    qs = QueryService(db_config=postgres_config)

    ob = ObjectBuilder(f"http://localhost:{sr.input_port}", kafka_config)
    channel = grpc.insecure_channel(f"localhost:{ob.input_port}")
    stub = object_builder_pb2_grpc.ObjectBuilderStub(channel)

    sr.start()
    qs.start()
    ob.start()

    yield viewId, stub, expected, expectedError

    ob.stop()
    qs.stop()
    sr.stop()

    # cleanup environmeFt
    delete_kafka_topic(kafka_config, TOPIC)
    clear_data(postgres_config)


def test_materialization(prepare):
    viewId, ob, expected, expectedError = prepare

    try:
        response = ob.Materialize(ViewId(view_id=viewId))

        response.rows.sort(key=lambda elem: elem.object_id)

        assert_json(MessageToDict(response), expected)
    except grpc.RpcError as rpc_error:
        error_str = f"{rpc_error}"
        if expectedError is not None:
            print(f"Error: `{error_str}`")
            print(f"Expected error: `{expectedError}`")
            assert error_str.find(expectedError) != -1
        else:
            assert False, f"Received error: {error_str}"
