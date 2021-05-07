```toml
communication_method = "kafka"
repository_kind = "postgres"
async_task_limit = 32

[notifications]
enabled = true
destination = ""

[postgres]
username = ""
password = ""
host = ""
port = ""
dbname = ""
schema = ""

[victoria_metrics]
url = ""

[druid]
topic = ""

[kafka]
brokers = ""
group_id = ""

[amqp]
exchange_url = ""
tag = ""

[amqp.consume_options]
no_local = false
no_act = false
exclusive = false
nowait = false

[grpc]
address = ""

[listener]
ordered_sources = [""]
unordered_sources = [""]

[monitoring]
metrics_port = 0
stats_port = 0
otel_service_name = ""

[log]
rust_log = "info,command_service=debug"

```
