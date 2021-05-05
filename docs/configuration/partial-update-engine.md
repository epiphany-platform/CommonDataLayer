```toml
communication_method = "kafka"
sleep_phase_length = 1000

[notification_consumer]
brokers = ""
group_id = ""
source = ""

[kafka]
brokers = ""
egest_topic = ""

[services]
schema_registry_url = "'"

[monitoring]
metrics_port = 0
stats_port = 0
otel_service_name = ""

[log]
rust_log = "info,partial_update_engine=debug"
```
