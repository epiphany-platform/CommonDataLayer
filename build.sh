#!/bin/bash
set -ex

# ENVS:
# Set only if you want to push images to remote repository
# CDL_REPOSITORY_PATH=epiphanyservices.azurecr.io/
# CDL_VERSION=0.1.0
# CDL_PUSH=true

array=( data-router command-service query-router query-service leader-elector schema-registry document-storage )
# shellcheck disable=SC2034
DOCKER_BUILDKIT=1


docker build -t workspace:local --build-arg ENV="${ENV:-PROD}" . -f workspace.Dockerfile

for i in "${array[@]}"
do
	docker build -t "${CDL_REPOSITORY_PATH}"cdl-"${i}":"${CDL_VERSION:-latest}" --build-arg BIN="${i}" . -f bin.Dockerfile --no-cache
done

if [[ -n "$CDL_PUSH" ]]
then
	for i in "${array[@]}"
	do
		docker push "${CDL_REPOSITORY_PATH}"cdl-"${i}":"${CDL_VERSION:-latest}"
	done
fi
