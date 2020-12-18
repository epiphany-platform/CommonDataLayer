#!/usr/bin/env bash

SCRIPT_DIR=$(dirname "$0")

if [ ! -f "registry_pb2.py" ]; then
  python3 -m grpc_tools.protoc --proto_path="$SCRIPT_DIR/../../../schema-registry/proto/" registry.proto --python_out=. --grpc_python_out=.
fi

python3 cli.py
