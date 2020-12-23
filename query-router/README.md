# Query Router

The REST interface over the CDL, used for retrieving data from any repository.

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
![../docs/graphs/QueryRouter-DataRetrieval.puml][query-router-puml]

[query-router-puml]: http://www.plantuml.com/plantuml/proxy?src=https://raw.githubusercontent.com/epiphany-platform/CommonDataLayer/develop/docs/graphs/query_router_data_retrieval.puml
[schema-registry]: ../schema-registry/README.md
[query-service]: ../query-service
[query-service-ts]: ../query-service-ts
[api-spec]: ./api.yml
