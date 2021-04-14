#!/usr/bin/env bash

docker-compose -f ../deployment/compose/docker-compose.yml \
	down --remove-orphans

docker-compose -f ../deployment/compose/docker-compose.yml \
	up -d \
	postgres \
	kafka \
	victoria_metrics
