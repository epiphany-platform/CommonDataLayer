# Getting Started

For infomration on specific services and their responsibilities:

- [Command Service][command-service]
- [Data Router][data-router]
- [Schema Registry][schema-registry]
- [Query Service][query-service]

# Installation

CDL is a written in Rust. See Rust's [installation][installation] guide to install. Below are the pre-requesites needed to get started: 

- Rust
- Docker
- Docker Compose

You can download [docker desktop][docker-desktop] for both Windows and MacOS to intall docker and docker compose on your local machine.

## Working with CDL Locally

Below is a following simple amount of steps to getting started working with the services in the CDL locally on your machine. To build and install container images of services within the CDL, run `build.sh` in root directory of this project.

Please review how to set up CDL locally on your machine but viewing [local setup][local-setup] documentations for a sample deployment. 

Below we will walk through a simple use case of the CDL:

**Use Case**
- Create Schema
- Insert Data
- Query Data

## Add Schema via CLI
A schema can be added through the CLI tool localed in the `cdl-cli` directory. To be able to run the cli you must have a rust compiler. The following command below creates the schema with a name according a json schema in a file as well as sets the topic for routing data through kafka. 

```
cargo run --bin cdl -- --registry-addr <registry_address> schema add --name <schema_name> --topic "cdl.document.input" --file <file_path_to_json>
```

Here is the sample JSON schema format that the CDL anticipates and ultimatley will validate data by. Please review [README][schema-registry] in `schema-registry` directory for more information.

```
{
	"$schema": "http://json-schema.org/draft-07/schema#",
    "$id": "http://example.com/product.schema.json",
	"definitions": {
		"1.0.0": {
            "description": "A work order",
            "type": "object",
            "properties": {
                "property1": {
                    "description": "",
                    "type": "integer"
                },
                "property2": {
                    "description":"",
                    "type": "string" 
                },
            },
            "required": ["property1"]
        }
    }
}
```

**NOTE**: Schema's can be added via [gRPC][grpc] to the schema registry. Ensure that you have `protoc` installed on your machine you machine generate [proto][proto] files in a supported language and make requests via a client.


## Insert Data

Data can be inserted into the system by data being written to Kafka or ingested through RabbitMQ. Data must be in JSON format with the following fields: `schemaId`, `objectId` and `data` to be routed through the CDL. 
It's worth noting that CDL doesn't rely on message key unless [message ordering][message-ordering] feature is enabled. However in order to keep system more performant it's advised to pass NULL as message key or evenly distributed strings.

Below is an example of the what input data would look like. Both ID fields are UUIDs.
```
{
    "schemaId": <UUID>,
    "objectId": <UUID>,
    "data": "{ \"some_propery": \"object\"}"

}
```

### Publish messae via RabbitMQ
Below is a sample `curl` command you can also publish a message through RabbitMQ web admin tool through a exchange or directly to a queue. Review [local setup][local-setup] for configuration details on Kafka, RabbitMQ.

The command below example takes input data and publishes to the default exchange in RabbitMQ. The message gets consumed and is sent to kafka and published to topic which is determined by `schemaId`.  The message is then routed to command service which handles routing and storage of data by type. 

```
curl -i -u ${user}:${pass} -H "Accept: application/json" \
-H "Content-Type:application/json" \
-XPOST -d'{"properties":{},"routing_key":"my_key","payload":"my body","payload_encoding":"string"}'\
http://${ampq_url}/api/exchanges/%2F/${exchange}/publish
```


## Query Data 

### Query via Query Service
Following this example local deployment, you can query for data saved. Here data is saved within the PR.
Ensure that environment variables are set for `POSTGRES_USERNAME`, `POSTGRES_PASSWORD`, `POSTGRES_HOST`, `POSTGRES_PORT`, `POSTGRES_DBNAME`, `POSTGRES_SCHEMA`, `INPUT_PORT` and `DS_QUERY_URL` or run query service directly on machine.

```
cargo run --bin query_service -- \
--schema-registry-addr <registry_addr> \
--ds-query-url <ds-url>  \
--input-port <input_port>
```


### Query Data via Query Router
The Query Router works with the query services to route requests to the correct repository, determined per schema (based on its query address).

```
cargo run --bin query_router -- \
--schema-registry-addr <schema_registry_addr> \
--cache-capacity <cache_capacity> \
--input-port <input_port>
```


## Deployment

See [chapter in the book][deployment]


[command-service]: ./architecture/command_service.md
[data-router]: ./architecture/data_router.md
[deployment]: ./deployment/index.md
[docker-desktop]: https://docs.docker.com/desktop/
[grpc]: https://grpc.io/docs/what-is-grpc/introduction/
[installation]: https://www.rust-lang.org/tools/install
[local-setup]: deployment/local/index.md
[proto]: https://github.com/epiphany-platform/CommonDataLayer/tree/develop/crates/rpc/proto
[query-service]: ./architecture/query_service.md
[schema-registry]: ./architecture/schema_registry.md
[message-ordering]: ./features/ordering.md
