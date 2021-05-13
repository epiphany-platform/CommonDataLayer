```toml
communication_method = "kafka"
input_port = 50101
import_file = ""
export_dir = ""

[postgres]
username = ""
password = ""
host = ""
port = ""
dbname = ""
schema = ""

[kafka]
brokers = ""

[amqp]
exchange_url = ""

[monitoring]
metrics_port = 0
stats_port = 0
otel_service_name = "schema-registry"

[log]
rust_log = "info,schema_registry=debug"
```
