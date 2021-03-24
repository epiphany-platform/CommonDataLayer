import grpc
import pytest

from tests.common.edge_registry import EdgeRegistry
from tests.common.kafka import KafkaInputConfig, create_kafka_topic, delete_kafka_topic
from tests.common.postgres import PostgresConfig, clear_relations
from tests.rpc.proto import edge_registry_pb2_grpc
from tests.rpc.proto.edge_registry_pb2 import SchemaRelation, Empty, RelationDetails

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

    yield stub

    er.stop()

    # cleanup environment
    delete_kafka_topic(kafka_config, TOPIC)
    clear_relations(postgres_config)


def test_inserting(prepare):
    parent_schema_id = "3fb03807-2c51-43c8-aa57-34f8d2fa0186"
    child_schema_id = "1d1cc7a5-9277-48bc-97d3-3d99cfb633dd"
    resp = prepare.AddRelation(SchemaRelation(parent_schema_id=parent_schema_id, child_schema_id=child_schema_id))
    relations = prepare.ListRelations(Empty())

    assert RelationDetails(relation_id=resp.relation_id, parent_schema_id=parent_schema_id,
                           child_schema_id=child_schema_id) in relations.items
