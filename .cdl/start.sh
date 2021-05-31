#!/usr/bin/env bash

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

export CDL_CONFIG="$SCRIPT_DIR/development"

cd "$SCRIPT_DIR/.." || exit 1
mkdir -p .tmp

cargo build --workspace || exit 1

./target/debug/data-router           > .tmp/data-router.log           2>&1 &
./target/debug/schema-registry       > .tmp/schema-registry.log       2>&1 &
./target/debug/edge-registry         > .tmp/edge-registry.log         2>&1 &
./target/debug/api                   > .tmp/api.log                   2>&1 &
./target/debug/command-service       > .tmp/command-service.log       2>&1 &
./target/debug/query-service         > .tmp/query-service.log         2>&1 &
./target/debug/materializer-general  > .tmp/materializer-general.log  2>&1 &
./target/debug/partial-update-engine > .tmp/partial-update-engine.log 2>&1 &
./target/debug/object-builder        > .tmp/object-builder.log        2>&1 &
