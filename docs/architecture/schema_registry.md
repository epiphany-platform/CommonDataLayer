# Schema Registry

### Technical Description

The Schema Registry (`SR` for short) is responsible for storing configuration about the data types handled by CDL. It is a persistent graph database, that can be queried via gRPC (other means of interaction are in progress). Currently there is no GUI nor TUI; user interaction is currently performed with the [CDL-CLI][CDL-CLI]. Replication across multiple instances of the Schema Registry is supported.

Interacts with:
- nothing on its own

Is used by:
- Data Router
- Query Router
- cdl-cli

Query methods:
- gRPC (clients may use cdl-cli CLI application)

Communication methods (supported repositores):
- Kafka (with other schema-registry instances)

### Configuration (Environment Variables)

| Name                 | Short Description                                                                                                | Example                       | Mandatory | Default |
|----------------------|------------------------------------------------------------------------------------------------------------------|-------------------------------|-----------|---------|
| INPUT_PORT           | Port to listen on                                                                                                | 50103                         | yes       |         |
| COMMUNICATION_METHOD | The method of communication with external services                                                               | `kafka` / `amqp` / `grpc`     | yes       |         |
| REPLICATION_ROLE     | (deprecated)                                                                                                     | `master` / `slave` / `none`   | yes       |         |
| DB_NAME              | Database name                                                                                                    | `schema-registry`             | yes       |         |
| POD_NAME             | (deprecated) used to promote to `master` role                                                                    | `schema1`                     | no        |         |
| EXPORT_DIR           | Directory to save state of the database. The state is saved in newly created folder with timestamp               | `/var/db`                     | no        |         |
| IMPORT_FILE          | JSON file from which SR should load initial state. If the state already exists this env variable will be ignored | `/var/db/initial-schema.json` | no        |         |
| METRICS_PORT         | Port to listen on for Prometheus requests                                                                        | 58105                         | no        | 58105   |
| RUST_LOG             | Log level                                                                                                        | `trace`                       | no        |         |

#### Kafka Configuration 
*(if `COMMUNICATION_METHOD` equals `kafka`)*

| Name           | Short Description        | Example           | Mandatory | Default |
|----------------|--------------------------|-------------------|-----------|---------|
| KAFKA_BROKERS  | Address of Kafka brokers | `kafka:9093`      | yes       |         |
| KAFKA_GROUP_ID | Group ID of the consumer | `schema_registry` | yes       |         |

#### AMQP Configuration 
*(if `COMMUNICATION_METHOD` equals `amqp`)*

| Name                   | Short Description             | Example                                  | Mandatory | Default |
|------------------------|-------------------------------|------------------------------------------|-----------|---------|
| AMQP_CONNECTION_STRING | Connection URL to AMQP Server | `amqp://user:CHANGEME@rabbitmq:5672/%2f` | yes       |         |
| AMQP_CONSUMER_TAG      | Consumer tag                  | `schema_registry`                        | yes       |         |

#### Replication Configuration 
*(if `COMMUNICATION_METHOD` does NOT equal `grpc`)*

| Name                    | Short Description         | Example                        | Mandatory | Default |
|-------------------------|---------------------------|--------------------------------|-----------|---------|
| REPLICATION_SOURCE      | Kafka topic/AMQP queue    | `cdl.schema_registry.internal` | yes       |         |
| REPLICATION_DESTINATION | Kafka topic/AMQP exchange | `cdl.schema_registry.internal` | yes       |         |

Mind that GRPC uses HTTP2 as its transport protocol (L4), so SCHEMA_REGISTRY_ADDR must be provided as `http://ip_or_name:port`


[CDL-CLI]: cli.md
