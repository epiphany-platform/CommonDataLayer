```toml
cache_capacity = 1000
input_port = 50103

[services]
schema_registry_url = ""

[monitoring]
metrics_port = 0
status_port = 0
otel_service_name = ""

[repositories]
key1 = { insert_destination = "", query_address = "", repository_type = "DocumentStorage" }
key2 = { insert_destination = "", query_address = "", repository_type = "Timeseries" }

[log]
rust_log = "info,query_router=debug"
```
