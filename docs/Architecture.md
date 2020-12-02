# Architecture of CDL

CommonDataLayer is cloud targeted system, oriented into allowing **user** to ingest *any* data, sort it and retrieve, *raw* or *mapped*.

CDL consists of four layers, each horizontally scalable and replaceable.

![./graphs/QueryRouter-DataRetrieval.puml](http://www.plantuml.com/plantuml/proxy?src=https://raw.githubusercontent.com/epiphany-platform/CommonDataLayer/develop/docs/graphs/CDL.puml)

## Configuration layer
Consists of services responsible for holding state and configuration of CDL.  
Currently only Schema Registry resides here. It keeps information about schemas and views. For more details please see [it's readme][schema-registry].

## Ingestion layer
Services in this layer are responsible for accepting generic messages from external systems via `Kafka`, validating them and sorting to correct repository.  
Currently consists only of [Data Router][data-router]. [Data Router][data-router] accepts messages in format:

```json
{
  "schemaID": "ca435cee-2944-41f7-94ff-d1b26e99ba48",
  "objectID": "fc0b95e1-07eb-4bf8-b691-1a85a49ef8f0",
  "payload": { ...valid json object }
}
```

for more details see [documentation][data-router]

## Storage layer
Consists of repositories for storing data.

We can distinguish 3 types of supported repositories:
- document
    - PostgreSQL
- blob
    - Internal solution using [Sled][sled] database
- timeseries
    - Victoria Metrics

### Command services
Service that translates messages received from [Data Router][data-router] into respective database format. Currently only one [Command Service][command-service] exists,
and is built in such way that it can support either one of databases.

### Query service
gRPC frontend of each database. Handles generic query format and translates it into DB query language.
Two query-services are present. One for timeseries databases, one for documents.

## Retrieval layer
Contains services responsible for materializing views and routing queries.
Query router is capable of retrieving data from various sources. More at [documentation][query-router].


[schema-registry]: ../schema-registry/README.md
[data-router]: ../data-router
[sled]: https://github.com/spacejam/sled
[command-service]: ../command-service
[query-router]: ../query-router/README.md
