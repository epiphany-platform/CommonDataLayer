# Query Router

### Technical Description

The Query Router (`QR`), is responsible for forwarding requests to specific query services. In CDL messages can be stored in any available repository, data router acts as a single entry point to multi-repo system and query router allows that data to be fetched easily.
Query Router first queries SR, then basing on received config, finds out specific QS that, hopefully, should be able to respond to specific query. Logic of that process is based on repo_type and query-service address stored with schema itself.


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

## Running
To run the **query-router** requires the [Schema Registry][schema-registry] to be running and the [Query Services][query-service] or the [Timeseries Query Services][query-service-ts] connected to their respective repositories.

Configuration is expected through the following environment variables:

| Name | Short Description | Example |
|---|---|---|
| SCHEMA_REGISTRY_ADDR | Address of setup schema registry | http://schema_registry:50101 |
| INPUT_PORT | Port to listen on | 50103 |
| CACHE_CAPACITY | How many entries the cache can hold | 1024 |

_Note: Currently, the cache is valid forever: changing a schema's **query-service** address will not update in the **query-router**._

## Functionality
REST API specification is available in [OpenAPI 3.0 spec][api-spec].

Currently, the **query-router** can:
- handle querying data by ID from document repositories,
- query range of data by ID from time series repositories,
- query data from repositories by SCHEMA_ID.

Rough sketch of working process:
``` plantuml
{{#include graphs/query_router_data_retrieval.puml}}
```

[schema-registry]: schema_registry.md
[query-service]: query_service.md
[query-service-ts]: https://github.com/epiphany-platform/CommonDataLayer/tree/develop/crates/query-service-ts
[api-spec]: https://github.com/epiphany-platform/CommonDataLayer/blob/develop/crates/query-router/api.yml
