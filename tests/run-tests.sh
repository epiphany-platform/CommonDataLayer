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

export WORKDIR=${SCRIPT_DIR}
export PYTHONPATH=$PYTHONPATH:"${SCRIPT_DIR}/../crates/"

python3 -m pytest "${SCRIPT_DIR}"
