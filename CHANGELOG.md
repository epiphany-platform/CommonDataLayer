# Common Data Layer Changelog

## Unreleased 
### Breaking changes

### Features
Added message batching to Data Router
Added querying and insertion to graphQL API
Added CDL Web Admin
Added custom Kafka acknowledgments
Added multithreading to Data Router with ordering enabled
Added GRPC as an alternative communication method alongside with Kafka and AMQP

### Fixes
Added extra tracing logs
Added delay period during exitting
Made multitasking opt-out

### Documentation
Added documentation in form of MDBook available on Github Pages
Added feature lists
Updated and described ENV variables in book
Documented all commandline arguments available in `--help`
Added information about versioning

### Dependencies
Bumped juniper_grahql to 0.2.3
Bumped jsonschema to 0.6.0
Bumped rand to 0.8.3
Bumped serde to 1.0.124
Bumped serde_json to 1.0.64
Bumped serde_yaml to 0.8.17
Bumped juniper_warp to 0.6.2
Bumped log to 0.4.14
Bumped env_logger to 0.8.3
Bumped lapin to 1.6.8
Bumped url to 2.2.1
Bumped thiserror to 1.0.24
Bumped futures-util to 0.3.13
Bumped kube to 0.46.1
Bumped futures to 0.3.13

### CI and infrastructure
Added CDL API to Docker Registry
Bumped actions/cache to v2.1.4
Added Github Action to automatically deploy MDBook to Github Pages
Simplified Dockerfile
Refactored local deployment, unified & updated HELM variables
Added rust security audit
Increased timeout for services to start

### Internal
Moved Druid Query Service
Restructured graphQL API

### Tests
Oraganized black-box testing
Added automated tests for HELM deployment

## v0.2.0

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
