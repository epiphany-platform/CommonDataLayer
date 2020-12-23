# Query Router

### Technical Description

The Query Router (`QR`), is responsible for finding specific query service. Since messages can be stored in any available repository, data router acts as a single entry point to multi-repo system. Query Router first queries SR, then basing on received config, finds out specific QS that, hopefully, should be able to respond to specific query. Logic of that process is based on repo_type and query-service address stored with schema itself.


### Communication

Interacts with:
- Query Service
- Schema Registry

Query methods:
- REST (request-response)

Communication protocols:
- gRPC with query-services (request-response)
- gRPC with schema-registry (request-response)

### Configuration

`
INPUT_PORT
SCHEMA_REGISTRY_ADDR
CACHE_CAPACITY
RUST_LOG
`

See an example [configuration][configuration] of deployment of data router and other services.

[grpc]: https://grpc.io/docs/what-is-grpc/introduction/
[proto]: ../query-service/proto/query.proto
[schema-registry]: ../schema-registry/README.md
[configuration]: ../examples/deploy/SETUP.md
[endpoints]: ../query-service/proto/query.proto
