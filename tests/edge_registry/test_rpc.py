import grpc
import pytest

from tests.common.edge_registry import EdgeRegistry
from tests.common.kafka import KafkaInputConfig, create_kafka_topic, delete_kafka_topic
from tests.common.postgres import PostgresConfig, clear_relations
from tests.rpc.proto import edge_registry_pb2_grpc
from tests.rpc.proto.edge_registry_pb2 import SchemaRelation, Empty, RelationDetails, RelationQuery, SchemaId, ObjectRelations, RelationIdQuery, Edge, ObjectIdQuery

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
    channel = grpc.insecure_channel(f"localhost:{er.rpc_port}")
    stub = edge_registry_pb2_grpc.EdgeRegistryStub(channel)

    er.start()

    yield stub

    er.stop()

    # cleanup environment
    delete_kafka_topic(kafka_config, TOPIC)
    clear_relations(postgres_config)


def test_add_relation(prepare):
    parent_schema_id = "3fb03807-2c51-43c8-aa57-34f8d2fa0186"
    child_schema_id = "1d1cc7a5-9277-48bc-97d3-3d99cfb633dd"
    resp = prepare.AddRelation(
        SchemaRelation(parent_schema_id=parent_schema_id,
                       child_schema_id=child_schema_id))
    relations = prepare.ListRelations(Empty())

    result = relations.items[0]

    assert len(relations.items) == 1
    assert result.relation_id == resp.relation_id
    assert result.parent_schema_id == parent_schema_id
    assert result.child_schema_id == child_schema_id


def test_get_relation(prepare):
    parent_schema_id = "3fb03807-2c51-43c8-aa58-3468d26a0186"
    child_schema_id = "1d1cc7a5-9277-48bc-97d3-3d99cfb633ac"
    relation_id = prepare.AddRelation(
        SchemaRelation(parent_schema_id=parent_schema_id,
                       child_schema_id=child_schema_id)).relation_id

    result = prepare.GetRelation(
        RelationQuery(relation_id=relation_id,
                      parent_schema_id=parent_schema_id)).child_schema_id

    assert child_schema_id == result


def test_get_relations(prepare):
    parent_schema_id = "1d1cc7a5-9277-48bc-97d3-3d99cfb63300"
    relation1 = prepare.AddRelation(
        SchemaRelation(parent_schema_id=parent_schema_id,
                       child_schema_id="1d1cc7a5-9277-48bc-97d3-3d99cfb63301")
    ).relation_id
    relation2 = prepare.AddRelation(
        SchemaRelation(parent_schema_id=parent_schema_id,
                       child_schema_id="1d1cc7a5-9277-48bc-97d3-3d99cfb63302")
    ).relation_id

    result = list(
        map(
            lambda x: x.relation_id,
            prepare.GetSchemaRelations(
                SchemaId(schema_id=parent_schema_id)).items))

    assert [relation1, relation2] == result


def test_add_get_edge(prepare):
    parent_schema_id = "1d1cc7a5-9277-48bc-97d3-3d99cfb63303"
    child_schema_id = "ed1cc7a5-9277-48bc-97d3-3d99cfb6330c"
    relation = prepare.AddRelation(
        SchemaRelation(parent_schema_id=parent_schema_id,
                       child_schema_id=child_schema_id)).relation_id

    parent_object_id = "1d1cc7a5-9277-48bc-97d3-3d99cfb633aa"
    child1 = "1d1cc7a5-9277-48bc-97d3-3d99cfb633a1"
    child2 = "1d1cc7a5-9277-48bc-97d3-3d99cfb633a2"

    prepare.AddEdges(
        ObjectRelations(relations=[
            Edge(relation_id=relation,
                 parent_object_id=parent_object_id,
                 child_object_ids=[child1, child2])
        ]))

    result = prepare.GetEdge(
        RelationIdQuery(relation_id=relation,
                        parent_object_id=parent_object_id)).child_object_ids

    assert result == [child1, child2]


def test_get_edges(prepare):
    relation1 = prepare.AddRelation(
        SchemaRelation(parent_schema_id="1d1cc7a5-9277-48bc-97d3-3d99cfb63000",
                       child_schema_id="1d1cc7a5-9277-48bc-97d3-3d99cfb63001")
    ).relation_id
    relation2 = prepare.AddRelation(
        SchemaRelation(parent_schema_id="1d1cc7a5-9277-48bc-97d3-3d99cfb6300c",
                       child_schema_id="1d1cc7a5-9277-48bc-97d3-3d99cfb6300a")
    ).relation_id

    parent = "1d1cc7a5-9277-48bc-97d3-3d99cfb63002"
    child1 = "1d1cc7a5-9277-48bc-97d3-3d99cfb63003"
    child2 = "1d1cc7a5-9277-48bc-97d3-3d99cfb63005"

    prepare.AddEdges(
        ObjectRelations(relations=[
            Edge(relation_id=relation1,
                 parent_object_id=parent,
                 child_object_ids=[child1]),
            Edge(relation_id=relation2,
                 parent_object_id=parent,
                 child_object_ids=[child2])
        ]))

    result = list(
        map(lambda x: (x.relation_id, x.child_object_ids),
            prepare.GetEdges(ObjectIdQuery(object_id=parent)).relations))

    assert [(relation1, [child1]), (relation2, [child2])] == result
