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
export OBJECT_BUILDER_EXE="../target/debug/object-builder"

echo "pip3 install -r '../requirements.txt'"
pip3 install -r "../requirements.txt"

echo "mkdir -p 'rpc/proto'"
mkdir -p "rpc/proto"

echo "python3 -m grpc.tools.protoc -I'../crates/rpc/proto' ..."

python3 -m grpc.tools.protoc -I"../crates/rpc/proto" \
  --python_out="./rpc/proto" \
  --grpc_python_out="./rpc/proto" \
  ../crates/rpc/proto/*.proto

touch "rpc/proto/__init__.py"
touch "rpc/__init__.py"

echo "python3 -m pytest . -vv"
export PYTHONPATH="${PYTHONPATH}:./rpc/proto" # https://github.com/protocolbuffers/protobuf/issues/1491

python3 -m pytest -vv "${1:-"."}"
