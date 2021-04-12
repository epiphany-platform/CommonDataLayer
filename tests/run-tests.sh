#!/usr/bin/env bash

echo "cargo build --workspace --manifest-path '../Cargo.toml'"
cargo build --workspace --manifest-path "../Cargo.toml"

export COMMAND_SERVICE_EXE="../target/debug/command-service"
export DB_SHRINKER_POSTGRES_EXE="../target/debug/db-shrinker-postgres"
export QUERY_ROUTER_EXE="../target/debug/query-router"
export SCHEMA_REGISTRY_EXE="../target/debug/schema-registry"
export QUERY_SERVICE_EXE="../target/debug/query-service"
export QUERY_SERVICE_TS_EXE="../target/debug/query-service-ts"
export EDGE_REGISTRY_EXE="../target/debug/edge-registry"

echo "pip3 install -r '../requirements.txt'"
pip3 install -r "../requirements.txt"

echo "mkdir -p 'rpc/proto'"
mkdir -p "rpc/proto"

echo "python3 -m grpc.tools.protoc -I'../crates/' ..."
python3 -m grpc.tools.protoc -I"../crates/" \
  --python_out="." \
  --grpc_python_out="." \
  rpc/proto/schema_registry.proto rpc/proto/query_service.proto rpc/proto/query_service_ts.proto \
  rpc/proto/generic.proto rpc/proto/edge_registry.proto

touch "rpc/proto/__init__.py"
touch "rpc/__init__.py"

echo "running postgres migration!"
PGPASSWORdD="1234" psql -u postgres -h localhost -d postgres -f crates/schema-registry/migrations/20210215170655_init-db.sql

echo "python3 -m pytest . -vv"
python3 -m pytest "." -vv
