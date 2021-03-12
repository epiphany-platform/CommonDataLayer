# GraphQL API

Server which provides `/graphql` and `/graphiql` routes for CDL management.
It is self-describing, interactive and easy to use way to manage your instance.

# Getting started on local machine (via docker-compose)

Check our [guide](../deployment/local/docker-compose.md) to see how to deploy API locally.

You can access interactive graphQL editor at http://localhost:50106/graphiql. It supports auto-completion, has built-in documentation explorer and history. 

Because our schema-registry in docker-compose is automatically initialized with some schemas, you can start making queries right away, like:

``` graphql
{
    schemas {
      id,
      definitions {
        version,
        definition
      },
      views {
        expression
      }
    }
}
```

### Configuration (Environment Variables)


| Name                 | Short Description                                  | Example                      | Mandatory | Default |
|----------------------|----------------------------------------------------|------------------------------|-----------|---------|
| INPUT_PORT           | Port to listen on                                  | 50103                        | yes       |         |
| SCHEMA_REGISTRY_ADDR | Address of schema registry gRPC API                | http://schema_registry:50101 | yes       |         |
| QUERY_ROUTER_ADDR    | Address of query router gRPC API                   | http://query_router:50101    | yes       |         |
| COMMUNICATION_METHOD | The method of communication with external services | `kafka` / `amqp` / `grpc`    | yes       |         |
| RUST_LOG             | Log level                                          | `trace`                      | no        |         |

#### Kafka Configuration
*(if `COMMUNICATION_METHOD` equals `kafka`)*

| Name               | Short Description                                  | Example             | Mandatory | Default |
|--------------------|----------------------------------------------------|---------------------|-----------|---------|
| KAFKA_BROKERS      | Address to Kafka brokers                           | `kafka:9093`        | yes       |         |
| KAFKA_GROUP_ID     | Group ID of the consumer                           | `postgres_command`  | yes       |         |
| REPORT_SOURCE      | Kafka topic on which API listens for notifications | `cdl.notifications` | yes       |         |
| INSERT_DESTINATION | Kafka topic to which API inserts new objects       | `cdl.data.input`    | yes       |         |

#### AMQP Configuration 
*(if `COMMUNICATION_METHOD` equals `amqp`)*

| Name                   | Short Description                                 | Example                                  | Mandatory | Default |
|------------------------|---------------------------------------------------|------------------------------------------|-----------|---------|
| AMQP_CONNECTION_STRING | Connection URL to AMQP Server                     | `amqp://user:CHANGEME@rabbitmq:5672/%2f` | yes       |         |
| AMQP_CONSUMER_TAG      | Consumer tag                                      | `postgres_command`                       | yes       |         |
| REPORT_SOURCE          | AMQP queue on which API listens for notifications | `cdl.notifications`                      | yes       |         |
| INSERT_DESTINATION     | AMQP exchange to which API inserts new objects    | `cdl.data.input`                         | yes       |         |

#### gRPC Configuration 
*(if `COMMUNICATION_METHOD` equals `grpc`)*

| Name               | Short Description                                     | Example                    | Mandatory | Default |
|--------------------|-------------------------------------------------------|----------------------------|-----------|---------|
| INSERT_DESTINATION | gRPC service address on which API inserts new objects | `http://data_router:50101` | yes       |         |
