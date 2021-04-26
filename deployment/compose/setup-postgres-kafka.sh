#!/usr/bin/env bash
set -euo pipefail

docker-compose down --remove-orphans
docker-compose -f docker-compose.yml up -d postgres kafka jaeger

sleep 15s

docker-compose \
    -f docker-compose.yml \
    -f docker-compose.cdl.kafka.base.yml \
    -f docker-compose.cdl.kafka.postgres.yml \
    up -d \
    schema_registry # object builder needs it to be running

sleep 1s

docker-compose \
    -f docker-compose.yml \
    -f docker-compose.cdl.kafka.base.yml \
    -f docker-compose.cdl.kafka.postgres.yml \
    up \
    schema_registry \
    postgres_query \
    postgres_command \
    postgres_materializer \
    web_api \
    query_router \
    data_router \
    partial_update_engine \
    object_builder \
    materializer_ondemand
