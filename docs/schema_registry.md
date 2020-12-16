# Schema Registry

### Technical Description

Schema Registy (also, `SR` for short) is responible for storing configuration about the data types handled by CDL. Schema Registry is a persistant graph database, that can be interfaced via grpc (other means of interaction are in progresss). Currently there is no GUI nor TUI. User interaction consist of internal tool [cdl-cli][cdl-cli]. There are also basic replication ability built-in.

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

See an example [configuration][configuration] of deployment of data router and other services.

[configuration]: ../examples/deploy/SETUP.md
[cdl-cli]: cdl-cli.md
