#!/usr/bin/env bash

SCRIPT_DIR=$(dirname "$0")

cargo build --workspace --manifest-path "${SCRIPT_DIR}/../Cargo.toml"

export COMMAND_SERVICE_EXE="${SCRIPT_DIR}/../target/debug/command-service"
export DB_SHRINKER_POSTGRES_EXE="${SCRIPT_DIR}/../target/debug/db-shrinker-postgres"
export QUERY_ROUTER_EXE="${SCRIPT_DIR}/../target/debug/query-router"
export SCHEMA_REGISTRY_EXE="${SCRIPT_DIR}/../target/debug/schema-registry"
export QUERY_SERVICE_EXE="${SCRIPT_DIR}/../target/debug/query-service"
export QUERY_SERVICE_TS_EXE="${SCRIPT_DIR}/../target/debug/query-service-ts"

pip3 install -r "${SCRIPT_DIR}/../requirements.txt"

mkdir -p rpc/proto
python3 -m grpc.tools.protoc -I"${SCRIPT_DIR}/../crates/" \
  --python_out="${SCRIPT_DIR}" \
  --grpc_python_out="${SCRIPT_DIR}" \
  rpc/proto/schema_registry.proto rpc/proto/query_service.proto rpc/proto/query_service_ts.proto rpc/proto/command_service.proto

python3 -m pytest "${SCRIPT_DIR}" -vv
