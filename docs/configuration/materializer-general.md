```toml
communication_method = "kafka"
input_port = 50203

[postgres]
username = ""
password = ""
host = ""
port = ""
dbname = ""
schema = ""

[monitoring]
metrics_port = 0
stats_port = 0
otel_service_name = ""

[log]
rust_log = "info,materializer_general=debug"
```
