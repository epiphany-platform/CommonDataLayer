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

### Configuration

Basic values used by SR

```
INPUT_PORT
DB_NAME
REPLICATION_ROLE
KAFKA_BROKERS
KAFKA_GROUP_ID
KAFKA_TOPICS
POD_NAME
RUST_LOG
```

Mind that GRPC uses HTTP2 as its transport protocol (L4), so SCHEMA_REGISTRY_ADDR must be provided as `http://ip_or_name:port`


[CDL-CLI]: cli.md
