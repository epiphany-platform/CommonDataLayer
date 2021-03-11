# Query Service
Each Query Service serves a common set of queries, and translates those into their respective database's query language.
Two query-services are present: one for timeseries databases, and one for documents.

### Technical Description

The query service (`QS` or for example for postgresql its `QSPG`), is responsible for querying data from specific repository. It offers two paths that can be accessed:
First path depends on type of repo

### Communication
Communication to query service is done through [gRPC][grpc] based on two [endpoints][proto] of querying for data by `SCHEMA_ID` or multiple `OBJECT_ID`s. Query service communicates with multiple databases such as postgresql, druid, victoria metrics. Query service also communicates with [schema registry][schema-registry]. 

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

### Configuration (Environment Variables)

| Name         | Short Description                         | Example | Mandatory | Default |
|--------------|-------------------------------------------|---------|-----------|---------|
| INPUT_PORT   | Port to listen on                         | 50103   | yes       |         |
| METRICS_PORT | Port to listen on for Prometheus requests | 58105   | no        | 58105   |
| RUST_LOG     | Log level                                 | `trace` | no        |         |

#### Postgres Configuration

| Name              | Short Description                | Example     | Mandatory | Default  |
|-------------------|----------------------------------|-------------|-----------|----------|
| POSTGRES_USERNAME | Username                         | `cdl`       | yes       |          |
| POSTGRES_PASSWORD | Password                         | `cdl1234`   | yes       |          |
| POSTGRES_HOST     | Host of the server               | `127.0.0.1` | yes       |          |
| POSTGRES_PORT     | Port on which the server listens | 5432        | yes       |          |
| POSTGRES_DBNAME   | Database name                    | `cdl`       | yes       |          |
| POSTGRES_SCHEMA   | SQL Schema available for service | `cdl`       | no        | `public` |

See an example [configuration][configuration] of deployment of data router and other services. 

[grpc]: https://grpc.io/docs/what-is-grpc/introduction/
[schema-registry]: schema_registry.md
[configuration]: ../deployment/index.md
[proto]: https://github.com/epiphany-platform/CommonDataLayer/tree/develop/crates/rpc/proto
