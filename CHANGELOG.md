# Common Data Layer Changelog

## Unreleased 

### Breaking changes
Archived document-storage crate
Archived blob-store crate

### Features
Added store type of schema in schema registry
Added RAW queries to postgres
Added RAW to query-router
Added VictoriaMetrics Support
Added check if Kafka/AMQP topic exists before inserting new schema in schema registry
Added GRPC alternative ingestion method to command-service
Added input batching for TimeSeries
Added GraphQL server - `api`
Added (opt-in) message ordering
Added input-output communication method selection from ENV vars and command-line arguments

### Fixes
Added non-default namespace support
Added string escape mechanism for schema query
Fixed building docker images in some environments

### Documentation
Updated documentation for basic services
Added UML diagrams of our services

### Dependencies
Bumped anyhow to 1.0.38
Bumped thiserror to 1.0.23
Bumped reqwest to 0.10.10
Bumped jsonschema to 0.4.3
Bumped rand to 0.8.2
Bumped lapin to 1.6.6
Bumped postgres to 0.19.0
Bumped serde to 1.0.119
Bumped serde_json to 1.0.61
Bumped serde_yaml to 0.8.15
Bumped futures to 0.3.12
Bumped futures_util to 0.3.12
Bumped envy to 0.4.2
Bumped uuid to 0.8.2
Bumped pbr to 1.0.4
Bumped test-case to 1.1.0
Bumped log to 0.4.13
Bumped rust toolchain to 30-12-2020

### CI and infrastructure
Improved performance of CI
Added DepdendaBot
Added cargo-deny to CI
Added component tests jobs for PRs
Added markdown link check

### Internal
Refactored internal message format
Moved gRPC definitions to `rpc` crate
Moved crates to separate folder `crates`
Splited helm component definitions
Moved benchmarking outside crates

### Tests
Added testcontainers for acceptance tests
Added component tests for query-router
Added component tests for query-service-ts
