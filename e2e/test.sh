#!/usr/bin/env bash
set -ex

## run from repo main dir

export DOCKER_BUILDKIT=1

docker build -f Dockerfile.e2e -t cdl-e2e:latest --build-arg ENV=DEV .

kubectl apply -f ./e2e/pod.yml
sleep 2
kubectl logs cdl-e2e --follow 
kubectl delete pod cdl-e2e

