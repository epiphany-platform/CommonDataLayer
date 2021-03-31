import grpc
import pytest

from tests.common.schema_registry import SchemaRegistry
from tests.common.object_builder import ObjectBuilder
from tests.common.kafka import KafkaInputConfig, create_kafka_topic, delete_kafka_topic
from tests.rpc.proto import object_builder_pb2_grpc
from tests.rpc.proto.object_builder_pb2 import ViewId, MaterializedView, RowDefinition

TOPIC = "cdl.object_builder.tests_data"


@pytest.fixture
def prepare(tmp_path):
    # declare environment
    kafka_config = KafkaInputConfig(TOPIC)

    # prepare environment
    create_kafka_topic(kafka_config, TOPIC)

    sr = SchemaRegistry(str(tmp_path), kafka_config.brokers)

    ob = ObjectBuilder(f"http://localhost:{sr.input_port}", kafka_config)
    channel = grpc.insecure_channel(f"localhost:{ob.input_port}")
    stub = object_builder_pb2_grpc.ObjectBuilderStub(channel)

    sr.start()
    ob.start()

    yield stub, sr

    ob.stop()

    # cleanup environment
    delete_kafka_topic(kafka_config, TOPIC)


def test_materialization(prepare):
    ob, sr = prepare

    view_id = "3fb03807-2c51-43c8-aa57-34f8d2fa0186"

    sr.

    resp = ob.Materialize(ViewId(view_id=view_id))

    print("resp = ", resp)

    assert false


# def test_add_relation(prepare):
#     parent_schema_id = "3fb03807-2c51-43c8-aa57-34f8d2fa0186"
#     child_schema_id = "1d1cc7a5-9277-48bc-97d3-3d99cfb633dd"
#     resp = prepare.AddRelation(
#         SchemaRelation(parent_schema_id=parent_schema_id,
#                        child_schema_id=child_schema_id))
#     relations = prepare.ListRelations(Empty())

#     result = relations.items[0]

#     assert len(relations.items) == 1
#     assert result.relation_id == resp.relation_id
#     assert result.parent_schema_id == parent_schema_id
#     assert result.child_schema_id == child_schema_id


# def test_get_relation(prepare):
#     parent_schema_id = "3fb03807-2c51-43c8-aa58-3468d26a0186"
#     child_schema_id = "1d1cc7a5-9277-48bc-97d3-3d99cfb633ac"
#     relation_id = prepare.AddRelation(
#         SchemaRelation(parent_schema_id=parent_schema_id,
#                        child_schema_id=child_schema_id)).relation_id

#     result = prepare.GetRelation(
#         RelationQuery(relation_id=relation_id,
#                       parent_schema_id=parent_schema_id)).child_schema_id

#     assert child_schema_id == result


# def test_get_relations(prepare):
#     parent_schema_id = "1d1cc7a5-9277-48bc-97d3-3d99cfb63300"
#     relation1 = prepare.AddRelation(
#         SchemaRelation(parent_schema_id=parent_schema_id,
#                        child_schema_id="1d1cc7a5-9277-48bc-97d3-3d99cfb63301")
#     ).relation_id
#     relation2 = prepare.AddRelation(
#         SchemaRelation(parent_schema_id=parent_schema_id,
#                        child_schema_id="1d1cc7a5-9277-48bc-97d3-3d99cfb63302")
#     ).relation_id

#     result = list(
#         map(
#             lambda x: x.relation_id,
#             prepare.GetSchemaRelations(
#                 SchemaId(schema_id=parent_schema_id)).items))

#     assert [relation1, relation2] == result


# def test_add_get_edge(prepare):
#     parent_schema_id = "1d1cc7a5-9277-48bc-97d3-3d99cfb63303"
#     child_schema_id = "ed1cc7a5-9277-48bc-97d3-3d99cfb6330c"
#     relation = prepare.AddRelation(
#         SchemaRelation(parent_schema_id=parent_schema_id,
#                        child_schema_id=child_schema_id)).relation_id

#     parent_object_id = "1d1cc7a5-9277-48bc-97d3-3d99cfb633aa"
#     child1 = "1d1cc7a5-9277-48bc-97d3-3d99cfb633a1"
#     child2 = "1d1cc7a5-9277-48bc-97d3-3d99cfb633a2"

#     prepare.AddEdges(
#         ObjectRelations(relations=[
#             Edge(relation_id=relation,
#                  parent_object_id=parent_object_id,
#                  child_object_ids=[child1, child2])
#         ]))

#     result = prepare.GetEdge(
#         RelationIdQuery(relation_id=relation,
#                         parent_object_id=parent_object_id)).child_object_ids

#     assert result == [child1, child2]


# def test_get_edges(prepare):
#     relation1 = prepare.AddRelation(
#         SchemaRelation(parent_schema_id="1d1cc7a5-9277-48bc-97d3-3d99cfb63000",
#                        child_schema_id="1d1cc7a5-9277-48bc-97d3-3d99cfb63001")
#     ).relation_id
#     relation2 = prepare.AddRelation(
#         SchemaRelation(parent_schema_id="1d1cc7a5-9277-48bc-97d3-3d99cfb6300c",
#                        child_schema_id="1d1cc7a5-9277-48bc-97d3-3d99cfb6300a")
#     ).relation_id

#     parent = "1d1cc7a5-9277-48bc-97d3-3d99cfb63002"
#     child1 = "1d1cc7a5-9277-48bc-97d3-3d99cfb63003"
#     child2 = "1d1cc7a5-9277-48bc-97d3-3d99cfb63005"

#     prepare.AddEdges(
#         ObjectRelations(relations=[
#             Edge(relation_id=relation1,
#                  parent_object_id=parent,
#                  child_object_ids=[child1]),
#             Edge(relation_id=relation2,
#                  parent_object_id=parent,
#                  child_object_ids=[child2])
#         ]))

#     result = list(
#         map(lambda x: (x.relation_id, x.child_object_ids),
#             prepare.GetEdges(ObjectIdQuery(object_id=parent)).relations))

#     assert [(relation1, [child1]), (relation2, [child2])] == result
