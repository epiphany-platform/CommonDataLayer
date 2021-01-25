# Query Service

### Technical Description

The query service (`QS` or for example for postgresql its `QSPG`), is responsible for querying data from specific repository. It offers two paths that can be accessed:
First path depends on type of repo

### Communication
Communication to query service is done through [gRPC][grpc] based on two [endpoints][endpoints] of querying for data by `SCHEMA_ID` or multiple `OBJECT_ID`s. Query service communicates with multiple databases such as postgresql, druid, sled. Query service also communicates with [schema registry][schema-registry]. 

Interacts with:
- Druid
- Postgresql
- VictoriaMetrics (accidentally also Prometheus)
- Sled
- Troika
- .. any similar grpc-able repo

Query methods:
- GRPC (req-response)

Communication protocols:
- database specific

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
[schema-registry]: ../crates/schema-registry/README.md
[configuration]: ../deployment/compose/README.md
[endpoints]: ../crates/rpc/proto
