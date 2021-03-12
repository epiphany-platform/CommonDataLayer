# Command Services
Services that translate messages received from the [Data Router][data-router] into their respective database's format. Currently only one Command Service implementation exists
and is built in such way that it can support multiple databases (one at a time).

### Technical Description

The Command-Service (commonly refered also as `CS`, or `CSPG` - indicating posgres instance), interfaces storage repositories with the CDL ecosystem.

Interacts with:
- Data Router (optional, either)
- Message Queue (optional, either)
- Supported Repository (one of)

Ingest methods:
- Kafka
- RabbitMq
- GRPC (currently either only one instance without kubernetes)

Egest methods (supported repositories):
- Postgresql (tested on 12, should support anything >=9, advised 13)
- VictoriaMetrics
- Druid
- Sleight (CDL's document storage)
- Troika (CDL's binary data repo)
- .. or anything with matching GRPC :)

### Configuration (Environment Variables)

| Name                 | Short Description                                                                               | Example                   | Mandatory | Default |
|----------------------|-------------------------------------------------------------------------------------------------|---------------------------|-----------|---------|
| COMMUNICATION_METHOD | The method of communication with external services                                              | `kafka` / `amqp` / `grpc` | yes       |         |
| REPORT_DESTINATION   | Kafka topic/AMQP exchange/callback URL to send notifications to (reporting disabled when empty) | `cdl.notifications`       | no        |         |
| METRICS_PORT         | Port to listen on for Prometheus requests                                                       | 58105                     | no        | 58105   |
| RUST_LOG             | Log level                                                                                       | `trace`                   | no        |         |

#### Postgres Configuration

| Name              | Short Description                | Example     | Mandatory | Default  |
|-------------------|----------------------------------|-------------|-----------|----------|
| POSTGRES_USERNAME | Username                         | `cdl`       | yes       |          |
| POSTGRES_PASSWORD | Password                         | `cdl1234`   | yes       |          |
| POSTGRES_HOST     | Host of the server               | `127.0.0.1` | yes       |          |
| POSTGRES_PORT     | Port on which the server listens | 5432        | yes       |          |
| POSTGRES_DBNAME   | Database name                    | `cdl`       | yes       |          |
| POSTGRES_SCHEMA   | SQL Schema available for service | `cdl`       | no        | `public` |

#### Druid Configuration

| Name                 | Short Description | Example                         | Mandatory | Default |
|----------------------|-------------------|---------------------------------|-----------|---------|
| DRUID_OUTPUT_BROKERS | Kafka brokers     | `kafka:9093`                    | yes       |         |
| DRUID_OUTPUT_TOPIC   | Kafka topic       | `cdl.timeseries.internal.druid` | yes       |         |

#### Victoria Metrics Configuration
| Name                        | Short Description           | Example                        | Mandatory | Default |
|-----------------------------|-----------------------------|--------------------------------|-----------|---------|
| VICTORIA_METRICS_OUTPUT_URL | Address of Victoria Metrics | `http://victoria_metrics:8428` | yes       |         |

#### Kafka Configuration 
*(if `COMMUNICATION_METHOD` equals `kafka`)*

| Name              | Short Description                | Example                    | Mandatory                                                                  | Default |
|-------------------|----------------------------------|----------------------------|----------------------------------------------------------------------------|---------|
| KAFKA_BROKERS     | Address of Kafka brokers         | `kafka:9093`               | yes                                                                        |         |
| KAFKA_GROUP_ID    | Group ID of the consumer         | `postgres_command`         | yes                                                                        |         |
| ORDERED_SOURCES   | Topics with ordered messages     | `cdl.timeseries.vm.1.data` | no, but one of `ORDERED_SOURCES` and `UNORDERED_SOURCES` has to be present |         |
| UNORDERED_SOURCES | Topics with unordered messages   | `cdl.timeseries.vm.2.data` | no, but one of `ORDERED_SOURCES` and `UNORDERED_SOURCES` has to be present |         |
| TASK_LIMIT        | Max requests handled in parallel | 32                         | yes                                                                        | 32      |

#### AMQP Configuration 
*(if `COMMUNICATION_METHOD` equals `amqp`)*

| Name                   | Short Description                | Example                                  | Mandatory                                                                  | Default |
|------------------------|----------------------------------|------------------------------------------|----------------------------------------------------------------------------|---------|
| AMQP_CONNECTION_STRING | Connection URL to AMQP Server    | `amqp://user:CHANGEME@rabbitmq:5672/%2f` | yes                                                                        |         |
| AMQP_CONSUMER_TAG      | Consumer tag                     | `postgres_command`                       | yes                                                                        |         |
| ORDERED_SOURCES        | Queues with ordered messages     | `cdl.timeseries.vm.1.data`               | no, but one of `ORDERED_SOURCES` and `UNORDERED_SOURCES` has to be present |         |
| UNORDERED_SOURCES      | Queues with unordered messages   | `cdl.timeseries.vm.2.data`               | no, but one of `ORDERED_SOURCES` and `UNORDERED_SOURCES` has to be present |         |
| TASK_LIMIT             | Max requests handled in parallel | 32                                       | yes                                                                        | 32      |

#### gRPC Configuration 
*(if `COMMUNICATION_METHOD` equals `grpc`)*

| Name                | Short Description            | Example               | Mandatory | Default |
|---------------------|------------------------------|-----------------------|-----------|---------|
|                     |                              |                       |           |         |
| GRPC_PORT           | Port to listen on            | 50103                 | yes       |         |
| REPORT_ENDPOINT_URL | URL to send notifications to | `notifications:50102` | yes       |         |

[data-router]: data_router.md
