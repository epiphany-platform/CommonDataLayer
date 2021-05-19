#!/usr/bin/env bash
set -euo pipefail

COMMUNICATION_METHOD=${1:-"kafka"}
REPOSITORY_KIND=${2:-"postgres"}

QS_TYPE="document"

if [ "$REPOSITORY_KIND" == "victoria_metrics" ] || [ "$REPOSITORY_KIND" == "druid" ]; then
  QS_TYPE="timeseries"
fi

echo "Starting $COMMUNICATION_METHOD + $QS_TYPE:$REPOSITORY_KIND"

sed -i '' -E "s|communication_method = .+|communication_method = \"$COMMUNICATION_METHOD\"|g" setup/configuration/default.toml
sed -i '' -E "s|repository_kind = .+|repository_kind = \"$REPOSITORY_KIND\"|g" setup/configuration/default.toml

docker-compose down --remove-orphans
docker-compose -f docker-compose.yml up -d postgres jaeger

sleep 15s

if [ "$COMMUNICATION_METHOD" == "kafka" ]; then
  docker-compose -f docker-compose.yml up -d kafka
elif [ "$COMMUNICATION_METHOD" == "amqp" ]; then
  docker-compose -f docker-compose.yml up -d rabbitmq
fi

if [ "$REPOSITORY_KIND" == "victoria_metrics" ]; then
  docker-compose -f docker-compose.yml up -d victoria_metrics
elif [ "$REPOSITORY_KIND" == "druid" ]; then
  docker-compose -f docker-compose.yml up -d \
    druid_coordinator druid_broker druid_historical druid_middlemanager druid_router
fi

sleep 5s

docker-compose \
  -f docker-compose.yml \
  up -d \
  schema_registry # object builder needs it to be running

sleep 1s

docker-compose \
  -f docker-compose.yml \
  up -d \
  schema_registry \
  command_service \
  web_api \
  query_router \
  data_router

if [ $QS_TYPE == "document" ]; then
  docker-compose -f docker-compose.yml up -d query_service
else
  docker-compose -f docker-compose.yml up -d query_service_ts
fi

if [ "$COMMUNICATION_METHOD" == "kafka" ]; then
  docker-compose -f docker-compose.yml up -d \
    object_builder \
    materializer_ondemand \
    materializer_general
fi
