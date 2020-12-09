# Query Service

### Technical Description

The query service (`QS` or for example for postgresql its `QSPG`), is responsible for querying data from specific repository. It offers two paths that can be accessed:
First path depends on type of repo

### Communication
Communication to query service is done through [gRPC][grpc] based on two [endpoints][endpoints] of querying for data by `SCHEMA_ID` or multiple `OBJECT_ID`s. Query service communicates with multiple databases such as postgresql, druid, sled. Query service also communicates with [schema registry][schema-registry]. 

Interacts with:
- Druid
- Postgresql
- VictoriaMetrics (accidentaly also Prometheus)
- Sled
- Troika
- .. any similar grpc-able repo

Ingest methods:
- GRPC (req-response)

Egest methods (supported repositores):
- GRPC

### Configuration

`
INPUT_PORT
RUST_LOG
POSTGRES_USERNAME
POSTGRES_PASSWORD
POSTGRES_HOST
POSTGRES_PORT
POSTGRES_DBNAME
POSTGRES_SCHEMA
`

See an example [configuration][configuration] of deployment of data router and other services. 

[grpc]: https://grpc.io/docs/what-is-grpc/introduction/
[proto]: ../query-service/proto/query.proto
[schema-registry]: ../schema-registry/README.md
[configuration]: ../examples/deploy/SETUP.md
[endpoints]: ../query-service/proto/query.proto
