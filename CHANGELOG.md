# Common Data Layer Changelog

## 0.3.1-rc (Unreleased)
### Breaking changes

### New Components
- Object Builder (OB)
- Materializer - ondemand (M-OD)
- Materializer - general (M-G)
- CDL Web Admin (GUI)
- Edge Registry - (ER)
- Partial Update Engine - (PUA)

### Changes to existing components:
- Schema Registry now uses Postgres Backend
- CDL can now work in ordered (messages) mode
- CDL now uses Open Telemetry
- DataRouter supports Message Batching

### General Changes:
- Object Builder is available in Graphql API
- ObjectBuilder supports empty filters
- Object Builder can use filters in queries
- Edge Registry is available in Graphql API
- Edge Registry notifications support added
- All services can be queried with --help to get full list of options with explanations

### Major Bugfixes
- DataRouter - It is now possible to swith to multi/single threaded behaviour

### Documentation:
- General Materialization documentation and how-to
- CDL versioning
- Updated ENV variables and their descriptions
- Add Feature List and descriptions
- GRPC implementation

### General Fixes
- Consumer-based services will log error and continue on invalid message. no longer resulting in crash loop
- Query multiple returns error for timeseries
- build script fix
- Metrics port defaulted to invalid ones

### RFCs:
- Materialization RFC
- RFC index page
- partial materialization
- edge repository
- Initial draft of the MessagePack
- Alternative communication method to Kafka and RabbitMQ

## v0.2.0

### Breaking changes
- Archived document-storage crate
- Archived blob-store crate

### Features
- Added store type of schema in schema registry
- Added RAW queries to postgres
- Added RAW to query-router
- Added VictoriaMetrics Support
- Added check if Kafka/AMQP topic exists before inserting new schema in schema registry
- Added GRPC alternative ingestion method to command-service
- Added input batching for TimeSeries
- Added GraphQL server - `api`
- Added (opt-in) message ordering
- Added input-output communication method selection from ENV vars and command-line arguments

### Fixes
- Added non-default namespace support
- Added string escape mechanism for schema query
- Fixed building docker images in some environments
-
### Documentation
- Updated documentation for basic services
- Added UML diagrams of our services
-
### Dependencies
- Bumped anyhow to 1.0.38
- Bumped thiserror to 1.0.23
- Bumped reqwest to 0.10.10
- Bumped jsonschema to 0.4.3
- Bumped rand to 0.8.2
- Bumped lapin to 1.6.6
- Bumped postgres to 0.19.0
- Bumped serde to 1.0.119
- Bumped serde_json to 1.0.61
- Bumped serde_yaml to 0.8.15
- Bumped futures to 0.3.12
- Bumped futures_util to 0.3.12
- Bumped envy to 0.4.2
- Bumped uuid to 0.8.2
- Bumped pbr to 1.0.4
- Bumped test-case to 1.1.0
- Bumped log to 0.4.13
- Bumped rust toolchain to 30-12-2020
-
### CI and infrastructure
- Improved performance of CI
- Added DepdendaBot
- Added cargo-deny to CI
- Added component tests jobs for PRs
- Added markdown link check
-
### Internal
- Refactored internal message format
- Moved gRPC definitions to `rpc` crate
- Moved crates to separate folder `crates`
- Splited helm component definitions
- Moved benchmarking outside crates
-
### Tests
- Added testcontainers for acceptance tests
- Added component tests for query-router
- Added component tests for query-service-ts
