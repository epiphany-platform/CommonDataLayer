```toml
communication_method = "kafka"
input_port = 50110

[postgres]
username = ""
password = ""
host = ""
port = ""
dbname = ""
schema = ""

[kafka]
brokers = ""
ingest_topic = ""
group_id = ""

[amqp]
exchange_url = ""
tag = ""
ingest_queue = ""

[amqp.consume_options]
no_local = false
no_act = false
exclusive = false
nowait = false

[notifications]
destination = ""
enabled = true

[monitoring]
metrics_port = 0
status_port = 0
otel_service_name = ""

[log]
rust_log = "info,edge_registry=debug"
```
