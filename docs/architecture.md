# Architecture of CDL

The Common Data Layer (CDL) is a cloud-native system, aimed at allowing **users** to store *any* data and retrieve it (*raw* or *mapped*).

The CDL consists of four layers, each horizontally scalable and replaceable.

![./graphs/cdl.puml][architecture-puml]

## Configuration Layer
Consists of services responsible for holding state and configuration of CDL.  
Currently only the Schema Registry resides here, which keeps information about schemas and views. For more details please see [its readme][schema-registry].

## Ingestion Layer
Services in this layer are responsible for accepting generic messages from external systems via `Kafka`, validating them and sorting to correct repository.  
Currently consists only of the [Data Router][data-router]. The [Data Router][data-router] accepts messages in the following format:

```json
{
  "schemaId": "ca435cee-2944-41f7-94ff-d1b26e99ba48",
  "objectId": "fc0b95e1-07eb-4bf8-b691-1a85a49ef8f0",
  "data": { ...valid json object }
}
```

For more details, see the Data Router's [readme][data-router].

## Storage Layer
Consists of repositories for storing data.

We can distinguish 3 types of supported repositories:
- Document
    - PostgreSQL
- Blob
    - Internal solution using [Sled][sled] database
- Timeseries
    - Victoria Metrics

### Command Services
Services that translate messages received from the [Data Router][data-router] into their respective database's format. Currently only one [Command Service][command-service] exists
and is built in such way that it can support multiple databases (one at a time).

### Query Service
Each Query Service serves a common set of queries, and translates those into their respective database's query language.
Two query-services are present: one for timeseries databases, and one for documents.

## Retrieval Layer
Contains services responsible for materializing views and routing queries.
The Query Router is capable of retrieving data from various repositories. More at [documentation][query-router].


[architecture-puml]: http://www.plantuml.com/plantuml/proxy?src=https://raw.githubusercontent.com/epiphany-platform/CommonDataLayer/develop/docs/graphs/cdl.puml
[schema-registry]: ../crates/schema-registry/README.md
[data-router]: ../crates/data-router
[sled]: https://github.com/spacejam/sled
[command-service]: ../crates/command-service
[query-router]: ../crates/query-router/README.md
