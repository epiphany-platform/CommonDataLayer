# Command Service

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

Egest methods (supported repositores):
- Postgresql (tested on 12, should support anything >=9, advised 13)
- VictoriaMetrics
- Druid
- Sleight (CDL's document storage)
- Troika (CDL's binary data repo)
- .. or anything with matching GRPC :)

### Configuration (environment files)

```
KAFKA_INPUT_GROUP_ID
KAFKA_INPUT_BROKERS
KAFKA_INPUT_TOPIC
POSTGRES_USERNAME
POSTGRES_PASSWORD
POSTGRES_HOST
POSTGRES_PORT
POSTGRES_DBNAME
POSTGRES_SCHEMA
SLEIGH_OUTPUT_ADDR
DRUID_OUTPUT_BROKER
DRUID_OUTPUT_TOPIC
REPORT_BROKER
REPORT_TOPIC
```
