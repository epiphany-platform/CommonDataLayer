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
backup_data = { insert_destination = "", query_address = "", repository_type = "DocumentStorage" }

[log]
rust_log = "info,query_router=debug"
```
