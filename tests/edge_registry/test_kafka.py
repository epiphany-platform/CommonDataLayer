import grpc
import pytest
import json
import time

from tests.common.edge_registry import EdgeRegistry
from tests.common.kafka import KafkaInputConfig, create_kafka_topic, delete_kafka_topic, push_to_kafka
from tests.common.postgres import PostgresConfig, clear_relations
from tests.rpc.proto import edge_registry_pb2_grpc
from tests.rpc.proto.edge_registry_pb2 import SchemaRelation, RelationIdQuery

TOPIC = "cdl.edge.tests_data"


@pytest.fixture
def prepare():
    # declare environment
    kafka_config = KafkaInputConfig(TOPIC)
    postgres_config = PostgresConfig()

    # prepare environment
    clear_relations(postgres_config)
    create_kafka_topic(kafka_config, TOPIC)

    er = EdgeRegistry(kafka_config, postgres_config)
    channel = grpc.insecure_channel(f"localhost:{er.communication_port}")
    stub = edge_registry_pb2_grpc.EdgeRegistryStub(channel)

    er.start()

    yield stub, kafka_config

    er.stop()

    # cleanup environment
    delete_kafka_topic(kafka_config, TOPIC)
    clear_relations(postgres_config)


def test_kafka(prepare):
    stub, kafka_config = prepare
    parent_schema_id = "3fb03807-2c51-43c8-aa57-34f8d2fa0186"
    child_schema_id = "1d1cc7a5-9277-48bc-97d3-3d99cfb633dd"
    relation_id = stub.AddRelation(
        SchemaRelation(parent_schema_id=parent_schema_id,
                       child_schema_id=child_schema_id)).relation_id

    parent_object_id = "1d1cc7a5-9277-48bc-97d3-3d99cfb63300"
    child_object_id = "1d1cc7a5-9277-48bc-97d3-3d99cfb63301"
    push_to_kafka(kafka_config, [{
        "relation_id": relation_id,
        "parent_object_id": parent_object_id,
        "child_object_ids": [child_object_id]
    }],
                  key='edge',
                  timestamp=1616759706)

    time.sleep(2)

    result = stub.GetEdge(
        RelationIdQuery(relation_id=relation_id,
                        parent_object_id=parent_object_id)).child_object_ids

    assert [child_object_id] == result
