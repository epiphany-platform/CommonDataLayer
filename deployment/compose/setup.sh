#!/usr/bin/env bash
set -euo pipefail

COMMUNICATION_METHOD=${1:-"kafka"}
REPOSITORY_KIND=${2:-"postgres"}

QS_TYPE="document"

if [ "$REPOSITORY_KIND" == "victoria_metrics" ] || [ "$REPOSITORY_KIND" == "druid" ]; then
  QS_TYPE="timeseries"
fi

echo "Starting $COMMUNICATION_METHOD + $QS_TYPE:$REPOSITORY_KIND"

sed -i -E "s|communication_method = .+|communication_method = \"$COMMUNICATION_METHOD\"|g" setup/configuration/default.toml
sed -i -E "s|repository_kind = .+|repository_kind = \"$REPOSITORY_KIND\"|g" setup/configuration/default.toml

docker-compose down --remove-orphans
docker-compose up -d postgres jaeger

sleep 5s

if [ "$COMMUNICATION_METHOD" == "kafka" ]; then
  docker-compose up -d kafka
elif [ "$COMMUNICATION_METHOD" == "amqp" ]; then
  docker-compose up -d rabbitmq
fi

if [ "$REPOSITORY_KIND" == "victoria_metrics" ]; then
  docker-compose up -d victoria_metrics
elif [ "$REPOSITORY_KIND" == "druid" ]; then
  docker-compose up -d \
    druid_coordinator druid_broker druid_historical druid_middlemanager druid_router
fi

sleep 15s

docker-compose up -d \
  schema_registry \
  command_service \
  web_api \
  query_router \
  data_router

if [ $QS_TYPE == "document" ]; then
  docker-compose up -d query_service
else
  docker-compose up -d query_service_ts
fi

if [ "$COMMUNICATION_METHOD" == "kafka" ]; then
  docker-compose up -d \
    object_builder \
    materializer_ondemand \
    materializer_general
fi
