#!/usr/bin/env bash

killall data-router           || echo "data-router"
killall schema-registry       || echo "schema-registry"
killall edge-registry         || echo "edge-registry"
killall api                   || echo "api"
killall command-service       || echo "command-service"
killall query-service         || echo "query-service"
killall materializer-general  || echo "materializer-general"
killall partial-update-engine || echo "partial-update-engine"
killall object-builder        || echo "object-builder"
