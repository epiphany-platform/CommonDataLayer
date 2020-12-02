# Query Router

REST interface over CDL. Used to retrieve data from repositories.

## Running
To run **query-router** requires [schema-registry][schema-registry] and [query-services][query-service] connected to repositories.

Environment required is provided via variables:

| name | short description | example |
|---|---|---|
| SCHEMA_REGISTRY_ADDR | Address of setup schema registry | http://schema_registry:50101 |
| INPUT_PORT | Port to listen on | 50103 |
| CACHE_CAPACITY | How many entries can cache hold | 1024 |

Please note that currently cache is valid forever - changes to schema **query-service** address will not be updated in **query-router**.

## Functionality
REST API specification is available in [OpenAPI 3.0 spec][api-spec].

Currently **query-router** can handle querying data by id from document repositories.

Rough sketch of working process:  
![../docs/graphs/QueryRouter-DataRetrieval.puml](http://www.plantuml.com/plantuml/proxy?src=https://raw.githubusercontent.com/epiphany-platform/CommonDataLayer/develop/docs/graphs/QueryRouter-DataRetrieval.puml)


[schema-registry]: ../schema-registry/README.md
[query-service]: ../query-service
[api-spec]: ./api.yml
