import grpc
import pytest
import json
import time

from tests.common.edge_registry import EdgeRegistry
from tests.common.kafka import KafkaInputConfig, create_kafka_topic, delete_kafka_topic, push_to_kafka
from tests.common.postgres import PostgresConfig, clear_relations
from tests.rpc.proto import edge_registry_pb2_grpc
from tests.rpc.proto.edge_registry_pb2 import SchemaRelation, RelationIdQuery

TOPIC = "cdl.object_builder.tests_data"
