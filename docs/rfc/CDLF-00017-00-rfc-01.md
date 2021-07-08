# Front Matter

```
Title           : Query raw routes
Author(s)       : Łukasz Biel
Team            : CommonDataLayer
Last updated    : 2021-07-06
Version         : 1.0.0
Notes           : Raw routes are already implemented
```

# Glossary
* `QR` - Query Router
* `QS` - Query Service
* `QS-ts` - Query Service - timeseries
* `gRPC` - Remote procedure call protocol - [website][gRPC]
* `VM` - Victoria Metrics

# Description
Raw routes in `QR` and `QS` allow users to bypass CDL api in places where it's lacking.

Accessing `raw` route is done via `/raw` endpoint in `QR`:
```yaml
{{#include ../../crates/query-router/api.yml:117:150}}
```
It's body is a json specific to targeted schema/repository, with schema:
```json
{ "raw_statement": "$ACTUAL_STATEMENT_WITHIN_JSON_STRING" }
```

`QS` and `QS-ts` have respective `gRPC` endpoints.

## Document repositories:
```protobuf
{{#include ../../crates/rpc/proto/query_service.proto:7}}
```
### PostgreSQL

`RawStatement` is an SQL query **string** that will get executed directly on the database, and result will be serialized into 2 dimensional string array.

> There is no safeguard for insert/delete statements, nor for config changes, however, using these isn't a good practice.
> User created for QS to access postgres should have those operations disabled.

## Timeseries repositories:
```protobuf
{{#include ../../crates/rpc/proto/query_service_ts.proto:7}}
```
### Victoria Metrics
"RawStatement" should be an escaped json with schema:
```json
{
  "method": "$METHOD",
  "endpoint": "$ENDPOINT",
  "queries": [["$KEY1", "$VALUE1"], ["$KEY2", "$VALUE2"]]
}
```
* `$ENDPOINT = /query_range | /query | /export | ...` - any available VictoriaMetrics endpoint  
* `$METHOD = GET | POST` - http method, only 2 are allowed, please refer to `VM` documentation for information when to use which.  
* `$KEY, $VALUE pairs` - request parameters, when `$METHOD == GET` they are translated to `query?` arguments, otherwise they are sent as a **FORM** body.

### Druid
"RawStatement" is a string sent directly to Druid instance as a body, via `POST` method.

# Additional work
We have to hide raw endpoints behind a feature flag. Such feature flag will be accessible from configuration tomls in the form of:
```toml
[features]
raw_endpoint = true|false
```
with default value set to true, as to not create breaking changes in 1.0 release. When 2.0 lands, we may consider turning this off by default.

Depending on this feature, `QR` may or may not expose `/raw` endpoint. There's no option for conditional exposition in query-services,
thus their routes must be available, just when `raw_endpoint = false` they will return Status code `Unavailable`.

[gRPC]: https://grpc.io/
