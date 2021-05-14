```toml
communication_method = "kafka"
input_port = 50107

[kafka]
brokers = ""
group_id = ""
ingest_topic = ""

[amqp]
exchange_url = ""
tag = ""
ingest_queue = ""

[amqp.consume_options]
no_local = false
no_act = false
exclusive = false
nowait = false

[services]
schema_registry_url = ""

[monitoring]
metrics_port = 0
status_port = 0
otel_service_name = ""

[log]
rust_log = "info,object_builder=debug"
```
