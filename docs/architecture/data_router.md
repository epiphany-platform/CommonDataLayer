# Data Router

### Technical Description

The data router (internally `DR` is also used) is responsible for taking in input data and routing it to the correct storage based on 
the data's schema and its associated topic. 

### Communication

The data router routes requests from RabbitMQ and Kafka to the correct storage solution based on the schema and data type.
Topic and some of the basic configuration is obtained from Schema Registry. Data are routed and deposited onto configured queues.

Interacts with:
- Command Service (optional, either)
- Message Queue (optional, either)
- Schema Registry

Ingest methods:
- Kafka

Internal communication methods:
- Kafka (command-service)
- gRPC (schema-registry)


Below are the example data required by data router:
```
# high level description
{
    "schemaId": <UUID>,
    "objectId": <UUID>,
    "data": { "some_property": "object"}
}

# type description
{
    "objectId"(string) : (128bit valid uuid),
    "schemaID"(string) : (128bit valid uuid),
    "data"(string) : (array,dict,object,string, literally anything),
}

# example, minimalistic one liner
{ "objectId": 9056c0b3-2ceb-42a6-a6b6-9718c3e273bc, "schemaId": 9056c0b3-2ceb-42a6-a6b6-9718c3e273bc, "data": {} }
```

Messages can be batched together, however please mind, that batched messages works best when used with the same schemaId.
Otherwise, messages will be split into sub-batches containing messages with the same schemaId
```
[
  { "objectId": 9056c0b3-2ceb-42a6-a6b6-9718c3e273bc, "schemaId": f79d7ebd-4260-4919-9ba3-45ea6701f065, "data": {} }
  { "objectId": 9056c0b3-2ceb-42a6-a6b6-9718c3e273bc, "schemaId": 9056c0b3-2ceb-42a6-a6b6-9718c3e273bc, "data": {} }
  { "objectId": 0369de4f-8025-4cf8-b6df-9446b51e4fd0, "schemaId": 9056c0b3-2ceb-42a6-a6b6-9718c3e273bc, "data": {} }
  { "objectId": 0369de4f-8025-4cf8-b6df-9446b51e4fd0, "schemaId": 07087162-e499-48f1-ad4a-cee7e77f1965, "data": {} }
]
```

Please mind that internally, each message will get its own timestamp, with which data started being processed by CDL. This information is invisible for user.


### Configuration (Environment Variables)

To configure the Data Router, set the following environment variables:


| Name                 | Short Description                                  | Example                      | Mandatory                       | Default |
|----------------------|----------------------------------------------------|------------------------------|---------------------------------|---------|
| COMMUNICATION_METHOD | The method of communication with external services | `kafka` / `amqp` / `grpc`    | yes                             |         |
| INPUT_SOURCE         | Kafka topic or AMQP queue                          | `cdl.data.input`             | no, when `grpc` has been chosen |         |
| SCHEMA_REGISTRY_ADDR | Address of schema registry gRPC API                | http://schema_registry:50101 | yes                             |         |
| CACHE_CAPACITY       | How many entries the cache can hold                | 1024                         | yes                             |         |
| TASK_LIMIT           | Max requests handled in parallel                   | 128                          | yes                             | 128     |
| METRICS_PORT         | Port to listen on for Prometheus requests          | 58105                        | no                              | 58105   |
| RUST_LOG             | Log level                                          | `trace`                      | no                              |         |

#### Kafka Configuration 
*(if `COMMUNICATION_METHOD` equals `kafka`)*

| Name           | Short Description        | Example       | Mandatory | Default |
|----------------|--------------------------|---------------|-----------|---------|
| KAFKA_BROKERS  | Address of Kafka brokers | `kafka:9093`  | yes       |         |
| KAFKA_GROUP_ID | Group ID of the consumer | `data_router` | yes       |         |

#### AMQP Configuration 
*(if `COMMUNICATION_METHOD` equals `amqp`)*

| Name                   | Short Description             | Example                                  | Mandatory | Default |
|------------------------|-------------------------------|------------------------------------------|-----------|---------|
| AMQP_CONNECTION_STRING | Connection URL to AMQP Server | `amqp://user:CHANGEME@rabbitmq:5672/%2f` | yes       |         |
| AMQP_CONSUMER_TAG      | Consumer tag                  | `data_router`                            | yes       |         |

#### gRPC Configuration 
*(if `COMMUNICATION_METHOD` equals `grpc`)*

| Name      | Short Description | Example | Mandatory | Default |
|-----------|-------------------|---------|-----------|---------|
| GRPC_PORT | Port to listen on | 50103   | yes       |         |

Mind that GRPC uses HTTP2 as its transport protocol (L4), so SCHEMA_REGISTRY_ADDR must be provided as `http://ip_or_name:port`

See an example [configuration][configuration] of deployment of data router and other services.

[configuration]: ../deployment/index.md
