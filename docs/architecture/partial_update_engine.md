# Partial Update Engine

## Configuration (Environment variables)

| Name                 | Short Description                         | Example                        | Mandatory   | Default |
|----------------------|-------------------------------------------|--------------------------------|-------------|---------|
| kafka_brokers        | Address of Kafka brokers                  | `kafka:9093`                   | yes         | no      |
| kafka_group_id       | Group ID of the consumer                  | `pue`                          | yes         | no      |
| notification_topic   | Kafka topic for notifications             | `3000`                         | yes         | no      |
| schema_registry_addr | Address of schema registry gRPC API       | `http://schema_registry:50101` | yes         | no      |
| metrics_port         | Port to listen on for Prometheus requests | `13456`                        | no(default) | `58105` |
| sleep_phase_length   | Duration of sleep phase in seconds        | `666`                          | yes         | no      |
