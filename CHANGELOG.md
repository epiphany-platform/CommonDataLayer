# Common Data Layer Changelog

## 1.0.0

### Breaking changes
- Data Router expects `version` field in CDL Ingestion Message in format "MAJOR_VERSION.MINOR_VERSION"
- Changed ENV variables

### New components
- CDL Web Admin (GUI) - For management purposes only
- Edge Registry (ER) - Manages relationships between schemas (schema to schema) and objects (object to object)
    - ER is available in graphQL API
    - ER pushes notifications
- Materializer - general (M-G) - Writes materialized data to the repository
- Materializer - ondemand (M-O) - Returns materialized data to the user (Request-Response)
- Object Builder (OB) - Creates materialized data based on first stage repositories
    - OB is available in graphQL API
    - OB supports empty filters for accessing all data
- Partial Update Engine (PUE) - Notifies OB when materialized view needs to be processed

### Changes to existing components
- Data Router and Query Router supports static routing allowing Schema-Registry-absent deployment
- Query Service supports returning stream of data without allocating hughe amounts of memory
- Schema Registry now uses Postgres as a backend
- Data Router supports message batching

### General changes
- E2E tests (E2E)
- Added gRPC as an alternative communication method between services
- CDL supports basic materialization with filtering
- CDL loads configuration from configuration files
- CDL supports Open Telemetry for traces

### Major bugfixes
- It is now possible to switch to multi/single threaded behaviour in Data Router

### Documentation
- Added MdBook on Github Pages
- Added Feature List and descriptions
- Added tutorial for basic materialization
- Added documentation for configuration files and for ENV variables
- Formalized commit messages and tags
- Added information about CDL versioning

### Geneal fixes
- Query-multiple RPC method returns an error for Timeseries schema type
- Metrics port defaulted to invalid one
- Build script fixes
- Consumer-based services will log an error and continue working on next message, no longer resulting in the crash loop

### RFCs
- Static routing
- CDL Ingestion API versioning
- CDL publishing deployment configurations
- Partial Materialization
- Edge Registry
- Alternative communication method to Kafka and RabbitMQ (gRPC)

### Known issues
- Relationships in materialization may generate wrong set of data

## v0.2.0

### Breaking changes
- Archived document-storage crate
- Archived blob-store crate

### Features
- Added store type of schema in Schema Registry
- Added RAW queries to postgres
- Added RAW to Query Router
- Added VictoriaMetrics Support
- Added check if Kafka/AMQP topic exists before inserting new schema in Schema Registry
- Added gRPC alternative ingestion method to Command Service
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
