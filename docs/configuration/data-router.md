```toml
communication_method = "kafka"
cache_capacity = 1000
async_task_limit = 32

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

[grpc]
address = ""

[repositories]
backup_data = { insert_destination = "", query_address = "", repository_type = "DocumentStorage" }

[monitoring]
metrics_port = 0
status_port = 0
otel_service_name = ""

[services]
schema_registry_url = ""

[log]
rust_log = ""
```
