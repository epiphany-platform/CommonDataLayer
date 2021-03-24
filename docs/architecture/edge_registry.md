# Edge registry

### Technical Description

Registry is responsible for storage of relations between schemas and objects.

### Communication

There are two methods of communicating with `ER` - gRPC and MessageQueue (RabbitMQ and Kafka are supported in this place).

#### gRPC communication

GRPC communication allows to access whole feature set of `ER` and is required for querying.
List of available commands can be found in registry's [proto file](../../crates/rpc/proto/edge_registry.proto).

#### Message queue communication

MQ currently serves as  an alternative means of ingestion for object relation data (called `edge` within registry).
Messages must follow JSON Schema:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "array",
  "items": [
    {
      "type": "object",
      "properties": {
        "relation_id": {
          "type": "string",
          "pattern": "[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}"
        },
        "parent_object_id": {
          "type": "string",
          "pattern": "[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}"
        },
        "child_object_ids": {
          "type": "array",
          "items": [
            {
              "type": "string",
              "pattern": "[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}"
            }
          ]
        }
      },
      "required": [
        "relation_id",
        "parent_object_id",
        "child_object_ids"
      ]
    }
  ]
}
```

eg.:

```json
[
    {
      "relation_id": "4d987502-8800-11eb-b5cb-0242ac130003",
      "parent_object_id": "79bbc2d5-92a6-43ad-b182-d6b9dd49184c",
      "child_object_ids": [
        "627f84c7-d9f0-4665-b54d-2fcb5422ce02", 
        "627f84c7-d9f0-4665-b54d-2fcb5422ce03"
      ]
    }
]
```

Each entry in top level array represents one-to-many relation within `relation_id`. 
Such `relation_id` should be added beforehand, via gRPC api, between objects schemas.
`ER` at this time does not validate correctness of inserted data, so it's up to user to ensure that `edges` and `relations` are configured properly.

### Configuration (Environment variables)

| Name               | Short Description                                 | Example           | Mandatory | Default  |
|--------------------|---------------------------------------------------|-------------------|-----------|----------|
| POSTGRES_USERNAME  |                                                   | postgres          | yes       |          |
| POSTGRES_PASSWORD  |                                                   | 1234qwer          | yes       |          |
| POSTGRES_HOST      |                                                   | 192.168.0.42      | yes       |          |
| POSTGRES_PORT      |                                                   | 5432              | no        | 5432     |
| POSTGRES_DBNAME    |                                                   | postgres          | yes       |          |
| POSTGRES_SCHEMA    |                                                   | cdl               | no        | postgres |
| COMMUNICATION_PORT | gRPC server port                                  | 50110             | no        | 50110    |
| METRICS_PORT       | Prometheus metrics port                           | 58105             | no        | 58105    |
| METHOD             | MQ ingestion method, can be `kafka` or `rabbitmq` | kafka             | yes       |          |
| MQ_HOST            | Kafka broker or RabbitMQ host                     | 192.168.0.51:9092 | yes       |          |
| MQ_TAG             | Kafka group_id or RabbitMQ tag                    | cdl_edge_registry | yes       |          |
| MQ_SOURCE          | Kafka topic or RabbitMQ queue                     | cdl.egde.input    | yes       |          |
